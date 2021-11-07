use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::pdns::common::PowerDnsPayload;
use crate::pdns::struct_type::StructType;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_id: StructType,
    url: String,
    kind: ZoneKind,
    rrsets: Vec<Rrset>,
    serial: u64,
    edited_serial: u64,
    masters: Vec<String>,
    dnssec: bool,
    nsec3param: String,
    nsec3narrow: bool,
    presigned: bool,
    soa_edit: String,
    soa_edit_api: String,
    api_rectify: bool,
    zone: Option<String>,
    account: Option<String>,
    nameservers: Vec<String>,
    master_tsig_key_ids: String,
    slave_tsig_key_ids: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewZone {
    name: String,
    #[serde(rename = "type")]
    type_id: StructType,
    kind: ZoneKind,
    rrsets: Vec<Rrset>,
    masters: Vec<String>,
    dnssec: bool,
    nsec3param: Option<String>,
    nsec3narrow: bool,
    presigned: bool,
    nameservers: Vec<String>,
    master_tsig_key_ids: Option<String>,
    slave_tsig_key_ids: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ZoneKind {
    Native,
    Master,
    Slave,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rrset {
    name: String,
    #[serde(rename = "type")]
    type_id: RrsetType,
    changetype: Option<Changetype>,
    records: Vec<Record>,
    comments: Vec<Comment>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RrsetType {
    A,
    #[serde(rename = "PTR")]
    Ptr,
    #[serde(rename = "AAAA")]
    Aaaa,
    #[serde(rename = "NS")]
    Ns,
    #[serde(rename = "SOA")]
    Soa,
    #[serde(rename = "CNAME")]
    Cname,
    #[serde(rename = "SRV")]
    Srv,
    #[serde(rename = "TXT")]
    Txt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Changetype {
    None,
    #[serde(rename = "REPLACE")]
    Replace,
    #[serde(rename = "DELETE")]
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    content: String,
    disabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    content: String,
    account: String,
    modified_at: u64,
}

impl PowerDnsPayload for Zone {}

impl NewZone {
    pub fn new(name: &String, rrsets: &Vec<Rrset>, masters: &Vec<String>,
               nameservers: &Vec<String>,
               dnssec: bool,
               nsec3param: Option<String>, nsec3narrow: bool,
               presigned: bool, master_tsig_key_ids: Option<String>,
               slave_tsig_key_ids: Option<String>) -> NewZone {
        if nameservers.is_empty() {
            NewZone {
                name: name.clone(),
                type_id: StructType::Zone,
                kind: ZoneKind::Slave,
                rrsets: Vec::new(),
                masters: masters.clone(),
                dnssec: dnssec,
                nsec3param: nsec3param,
                nsec3narrow: nsec3narrow,
                presigned: presigned,
                nameservers: Vec::new(),
                master_tsig_key_ids: master_tsig_key_ids,
                slave_tsig_key_ids: slave_tsig_key_ids,
            }
        } else {
            NewZone {
                name: name.clone(),
                type_id: StructType::Zone,
                kind: ZoneKind::Native,
                rrsets: rrsets.clone(),
                masters: Vec::new(),
                dnssec: dnssec,
                nsec3param: nsec3param,
                nsec3narrow: nsec3narrow,
                presigned: presigned,
                nameservers: nameservers.clone(),
                master_tsig_key_ids: master_tsig_key_ids,
                slave_tsig_key_ids: slave_tsig_key_ids,
            }
        }
    }
}

impl Rrset {
    pub fn new(name: &String,
               type_id: RrsetType,
               changetype: &Option<Changetype>,
               records: &Vec<Record>,
               comments: &Vec<Comment>) -> Rrset {
        Rrset {
            name: name.clone(),
            type_id: type_id,
            changetype: changetype.clone(),
            records: records.clone(),
            comments: comments.clone(),
        }
    }
}

impl Comment {
    pub fn new(content: &String, account: &String) -> Comment {
        let stamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(value) => value.as_secs(),
            Err(_) => 0 as u64
        };

        Comment {
            content: content.clone(),
            account: account.clone(),
            modified_at: stamp,
        }
    }
}

impl Record {
    pub fn new(content: &String, disabled: bool) -> Record {
        Record {
            content: content.clone(),
            disabled,
        }
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut rrsets: Vec<String> = Vec::new();

        for rrset in &self.rrsets {
            rrsets.push(format!("{}", rrset));
        }

        write!(f, "(id: {}, name: {}, type: {}, kind: {}, url: {}, rrsets: ({}), serial: {},  edited_serial: {}, masters: ({}), dnssec: {}, nsec3param: {}, nsec3narrow: {}, presigned: {}, soa_edit: {}, soa_edit_api: {}, api_rectify: {}, zone: {}, account: {}, nameservers: ({}), master_tsig_key_ids: {}, slave_tsig_key_ids: {})",
               &self.id, &self.name, &self.type_id, &self.kind, &self.url,
               rrsets.join(", "), self.serial, self.edited_serial,
               &self.masters.join(", "), self.dnssec, &self.nsec3param, self.nsec3narrow,
               self.presigned, &self.soa_edit, &self.soa_edit_api, self.api_rectify,
               &self.zone.clone().unwrap_or(String::new()),
               &self.account.clone().unwrap_or(String::new()),
               &self.nameservers.join(", "), &self.master_tsig_key_ids,
               &self.slave_tsig_key_ids)
    }
}

impl Display for NewZone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut rrsets: Vec<String> = Vec::new();

        for rrset in &self.rrsets {
            rrsets.push(format!("{}", rrset));
        }

        write!(f, "(name: {}, type: {}, kind: {}, rrsets: ({}), masters: ({}), dnssec: {}, nsec3param: {}, nsec3narrow: {}, presigned: {}, nameservers: ({}), master_tsig_key_ids: {}, slave_tsig_key_ids: {})",
               &self.name, &self.type_id, &self.kind, rrsets.join(", "),
               &self.masters.join(", "), self.dnssec,
               &self.nsec3param.clone().unwrap_or(String::new()), self.nsec3narrow,
               self.presigned, &self.nameservers.join(", "),
               &self.master_tsig_key_ids.clone().unwrap_or(String::new()),
               &self.slave_tsig_key_ids.clone().unwrap_or(String::new()))
    }
}

impl Display for ZoneKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ZoneKind::Master => write!(f, "Master"),
            ZoneKind::Native => write!(f, "Native"),
            ZoneKind::Slave => write!(f, "Slave"),
        }
    }
}

impl Display for Rrset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(name: {}, type: {}, changetype: {})",
               &self.name, &self.type_id,
               &self.changetype.clone().unwrap_or(Changetype::None))
    }
}

impl Display for RrsetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RrsetType::A => write!(f, "A"),
            RrsetType::Aaaa => write!(f, "AAAA"),
            RrsetType::Cname => write!(f, "CNAME"),
            RrsetType::Ns => write!(f, "NS"),
            RrsetType::Ptr => write!(f, "PTT"),
            RrsetType::Soa => write!(f, "SOA"),
            RrsetType::Srv => write!(f, "SRV"),
            RrsetType::Txt => write!(f, "TXT"),
        }
    }
}

impl Display for Changetype {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Changetype::Replace => write!(f, "REPLACE"),
            Changetype::Delete => write!(f, "DELETE"),
            Changetype::None => write!(f, "none"),
        }
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(content: {}, disabled: {})", &self.content, self.disabled)
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(content: {}, account: {}, modified_at {})",
               &self.content, &self.account, self.modified_at)
    }
}