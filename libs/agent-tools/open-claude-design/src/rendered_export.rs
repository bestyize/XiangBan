//! Bounded rendered output received only from a trusted desktop window.
use base64::Engine;
use serde::Deserialize;
use std::io::{Cursor,Read};

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct RenderedExport {pub revision:u64,pub data:String,pub file:Option<String>}

pub fn decode(format:&str,data:&str)->Result<Vec<u8>,String>{
    let prefix=match format {"png"=>"data:image/png;base64,","pdf"=>"data:application/pdf;base64,","pptx"=>"data:application/vnd.openxmlformats-officedocument.presentationml.presentation;base64,","fig"=>"data:application/x-figma;base64,",_=>return Err("不支持的渲染导出格式".into())};
    let encoded=data.strip_prefix(prefix).ok_or("导出数据格式不匹配")?;
    if encoded.len()>64*1024*1024{return Err("导出文件超过 48MiB".into());}
    let bytes=base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|_|"导出数据编码无效")?;
    if format=="png" {
        if bytes.len()<33||!bytes.starts_with(b"\x89PNG\r\n\x1a\n")||&bytes[12..16]!=b"IHDR"{return Err("PNG 数据无效".into());}
        let w=u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64;
        let h=u32::from_be_bytes(bytes[20..24].try_into().unwrap()) as u64;
        if w==0||h==0||w*h>16777216{return Err("PNG 尺寸超过限制".into());}
    }else if format=="pdf" {
        if !bytes.starts_with(b"%PDF-")||!bytes.windows(5).rev().take(64).any(|b|b==b"%%EOF"){return Err("PDF 数据无效".into());}
    }else{validate_archive(format,&bytes)?;}
    Ok(bytes)
}

fn validate_archive(format:&str,bytes:&[u8])->Result<(),String>{
    let mut archive=zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_|"导出文件不是有效的 ZIP 容器")?;
    if archive.len()>4096{return Err("导出容器条目过多".into());}
    let mut total=0u64;
    for i in 0..archive.len(){
        let file=archive.by_index(i).map_err(|_|"无法读取导出容器")?;
        total=total.checked_add(file.size()).ok_or("导出容器过大")?;
        if total>128*1024*1024||file.size()>64*1024*1024{return Err("导出容器解压大小超过限制".into());}
        let name=file.name().map_err(|_|"导出容器文件名无效")?;
        if file.enclosed_name().is_none()||name.contains('\\'){return Err("导出容器路径无效".into());}
        if name.to_ascii_lowercase().contains("vbaproject"){return Err("不支持含宏的导出文件".into());}
    }
    if format=="pptx"{
        for name in ["[Content_Types].xml","ppt/presentation.xml","ppt/slides/slide1.xml"]{
            if archive.by_name(name).is_err(){return Err("PowerPoint 文件缺少必要内容".into());}
        }
    }else{
        let mut canvas=archive.by_name("canvas.fig").map_err(|_|"Figma 文件缺少画布")?;
        let mut header=[0u8;12];canvas.read_exact(&mut header).map_err(|_|"Figma 画布已截断")?;
        if &header[..8]!=b"fig-kiwi"{return Err("Figma 画布编码无效".into());}
        for _ in 0..2{
            let mut size=[0u8;4];canvas.read_exact(&mut size).map_err(|_|"Figma 数据块已截断")?;
            let len=u32::from_le_bytes(size) as u64;
            if len==0||len>64*1024*1024{return Err("Figma 数据块大小无效".into());}
            let read=std::io::copy(&mut canvas.by_ref().take(len),&mut std::io::sink()).map_err(|_|"Figma 数据块无法读取")?;
            if read!=len{return Err("Figma 数据块已截断".into());}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_mismatched_truncated_and_oversized_images(){
        assert!(decode("pdf","data:image/png;base64,AA==").is_err());
        assert!(decode("png","data:image/png;base64,AA==").is_err());
        let mut png=vec![0;33];png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&1280u32.to_be_bytes());png[20..24].copy_from_slice(&800u32.to_be_bytes());
        let encode=|b:&[u8]|format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(b));
        assert_eq!(decode("png",&encode(&png)).unwrap(),png);
        png[16..20].copy_from_slice(&u32::MAX.to_be_bytes());assert!(decode("png",&encode(&png)).is_err());
    }
    #[test]
    fn accepts_pdf_only_with_header_and_trailer(){
        let data=b"%PDF-1.4\n1 0 obj\nendobj\n%%EOF\n";
        let encoded=format!("data:application/pdf;base64,{}",base64::engine::general_purpose::STANDARD.encode(data));
        assert_eq!(decode("pdf",&encoded).unwrap(),data);
        assert!(decode("pdf","data:application/pdf;base64,YWJj").is_err());
    }
    #[test]
    fn validates_pptx_and_fig_containers_and_rejects_renamed_archives(){
        use std::io::Write;
        let zip=|files:Vec<(&str,Vec<u8>)>|{let mut writer=zip::ZipWriter::new(Cursor::new(Vec::new()));for(name,data)in files{writer.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();writer.write_all(&data).unwrap();}writer.finish().unwrap().into_inner()};
        let pptx=zip(vec![("[Content_Types].xml",b"xml".to_vec()),("ppt/presentation.xml",b"xml".to_vec()),("ppt/slides/slide1.xml",b"xml".to_vec())]);
        assert!(validate_archive("pptx",&pptx).is_ok());assert!(validate_archive("fig",&pptx).is_err());
        let mut canvas=b"fig-kiwi".to_vec();canvas.extend(101u32.to_le_bytes());for _ in 0..2{canvas.extend(4u32.to_le_bytes());canvas.extend(b"data");}
        assert!(validate_archive("fig",&zip(vec![("canvas.fig",canvas.clone())])).is_ok());
        canvas.pop();assert!(validate_archive("fig",&zip(vec![("canvas.fig",canvas)])).is_err());
        assert!(validate_archive("pptx",&zip(vec![("../escape",vec![1])])).is_err());
        assert!(validate_archive("pptx",b"not a zip").is_err());
    }
}
