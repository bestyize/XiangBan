//! Bounded rendered output received only from a trusted desktop window.
use base64::Engine;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct RenderedExport {pub revision:u64,pub data:String,pub file:Option<String>}

pub fn decode(format:&str,data:&str)->Result<Vec<u8>,String>{
    let prefix=match format {"png"=>"data:image/png;base64,","pdf"=>"data:application/pdf;base64,",_=>return Err("不支持的渲染导出格式".into())};
    let encoded=data.strip_prefix(prefix).ok_or("导出数据格式不匹配")?;
    if encoded.len()>64*1024*1024{return Err("导出文件超过 48MiB".into());}
    let bytes=base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|_|"导出数据编码无效")?;
    if format=="png" {
        if bytes.len()<33||!bytes.starts_with(b"\x89PNG\r\n\x1a\n")||&bytes[12..16]!=b"IHDR"{return Err("PNG 数据无效".into());}
        let w=u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64;
        let h=u32::from_be_bytes(bytes[20..24].try_into().unwrap()) as u64;
        if w==0||h==0||w*h>16777216{return Err("PNG 尺寸超过限制".into());}
    }else if !bytes.starts_with(b"%PDF-")||!bytes.windows(5).rev().take(64).any(|b|b==b"%%EOF"){return Err("PDF 数据无效".into());}
    Ok(bytes)
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
}
