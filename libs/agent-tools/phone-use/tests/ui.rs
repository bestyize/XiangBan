use agent_phone_use_tools::ui::{target,Selector};

#[test]
fn duplicated_obfuscated_ids_require_a_semantic_selector() {
    let xml="<hierarchy><node resource-id=\"app:id/obfuscated\" content-desc=\"发布\" enabled=\"true\" clickable=\"true\" bounds=\"[488,2458][732,2605]\"/><node resource-id=\"app:id/obfuscated\" content-desc=\"搜索\" enabled=\"true\" clickable=\"true\" bounds=\"[10,10][30,30]\"/></hierarchy>";
    assert!(target(xml,&Selector {resource_id:Some("app:id/obfuscated".into()),..Default::default()}).unwrap_err().contains("2 nodes"));
    let hit=target(xml,&Selector {description:Some("发布".into()),..Default::default()}).unwrap();
    assert_eq!((hit.x,hit.y),(610,2531));
}

#[test]
fn label_taps_nearest_clickable_parent_and_unescapes_entities() {
    let xml="<hierarchy><node clickable=\"true\" enabled=\"true\" bounds=\"[10,20][210,120]\"><node text=\"A &amp; B\" enabled=\"true\" clickable=\"false\" bounds=\"[20,30][100,80]\"/></node></hierarchy>";
    let hit=target(xml,&Selector {text:Some("A & B".into()),..Default::default()}).unwrap();
    assert_eq!(hit.bounds,[10,20,210,120]);
}

#[test]
fn disabled_hidden_empty_and_offscreen_targets_do_not_produce_coordinates() {
    for attributes in ["enabled=\"false\" bounds=\"[0,0][10,10]\"","visible-to-user=\"false\" bounds=\"[0,0][10,10]\"","bounds=\"[0,0][0,10]\"","bounds=\"[-1,0][10,10]\"","bounds=\"[0,0][20000,10]\""] {
        let xml=format!("<hierarchy><node text=\"target\" clickable=\"true\" {attributes}/></hierarchy>");
        assert!(target(&xml,&Selector {text:Some("target".into()),..Default::default()}).is_err(),"{attributes}");
    }
    assert!(target("<hierarchy/>",&Selector::default()).is_err());
}

#[test]
fn class_refines_an_ambiguous_text_match() {
    let xml="<hierarchy><node text=\"全部\" class=\"TextView\" clickable=\"true\" bounds=\"[0,0][10,10]\"/><node text=\"全部\" class=\"Button\" clickable=\"true\" bounds=\"[20,20][30,30]\"/></hierarchy>";
    let selector=Selector {text:Some("全部".into()),class:Some("Button".into()),..Default::default()};
    assert_eq!(target(xml,&selector).unwrap().x,25);
}

#[test]
fn arbitrary_document_entities_are_rejected() {
    let xml="<!DOCTYPE hierarchy [<!ENTITY bad SYSTEM 'file:///private'>]><hierarchy><node text=\"&bad;\"/></hierarchy>";
    assert!(target(xml,&Selector {text:Some("anything".into()),..Default::default()}).is_err());
}
