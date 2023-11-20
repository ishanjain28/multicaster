#[derive(Debug, Default)]
pub struct ResourceRecord {
    name: String,
    rtype: u16,
    class: u16,
    ttl: u16,
    rdlength: u16,
    rdata: (),
}
