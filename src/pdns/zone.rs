// Copyright 2021 Cumulus Cloud Software und Consulting GmbH & Co KG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::pdns::common::PowerDnsPayload;
use crate::pdns::struct_type::StructType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_id: Option<StructType>,
    url: String,
    kind: ZoneKind,
    rrsets: Vec<Rrset>,
    serial: u64,
    edited_serial: u64,
    masters: Vec<String>,
    dnssec: bool,
    nsec3param: String,
    nsec3narrow: bool,
    presigned: Option<bool>,
    soa_edit: String,
    soa_edit_api: String,
    api_rectify: bool,
    zone: Option<String>,
    account: Option<String>,
    nameservers: Option<Vec<String>>,
    master_tsig_key_ids: Vec<String>,
    slave_tsig_key_ids: Vec<String>,
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
    master_tsig_key_ids: Option<Vec<String>>,
    slave_tsig_key_ids: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListZone {
    id: String,
    name: String,
    url: String,
    kind: ZoneKind,
    serial: u64,
    edited_serial: u64,
    notified_serial: u64,
    last_check: u64,
    masters: Vec<String>,
    dnssec: bool,
    account: Option<String>,
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
    ttl: Option<u32>,
    changetype: Option<Changetype>,
    records: Vec<Record>,
    comments: Vec<Comment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rrsets {
    rrsets: Vec<Rrset>,
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

#[allow(dead_code)]
impl Zone {
    pub fn rrsets(&self) -> Vec<Rrset> {
        self.rrsets.clone()
    }
}

impl NewZone {
    pub fn new(name: &String, rrsets: &Vec<Rrset>, masters: &Vec<String>,
               nameservers: &Vec<String>,
               dnssec: bool,
               nsec3param: Option<String>, nsec3narrow: bool,
               presigned: bool, master_tsig_key_ids: Option<Vec<String>>,
               slave_tsig_key_ids: Option<Vec<String>>) -> NewZone {
        if nameservers.is_empty() {
            NewZone {
                name: canonicalize_name(name),
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
                name: canonicalize_name(name),
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

#[allow(dead_code)]
impl Rrset {
    pub fn new(name: &String,
               type_id: RrsetType,
               changetype: &Option<Changetype>,
               ttl: &Option<u32>,
               records: &Vec<Record>,
               comments: &Vec<Comment>) -> Rrset {
        Rrset {
            name: name.clone(),
            type_id: type_id,
            changetype: changetype.clone(),
            ttl: ttl.clone(),
            records: records.clone(),
            comments: comments.clone(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn records(&self) -> Vec<Record> {
        self.records.clone()
    }
}

impl Rrsets {
    pub fn new(rrsets: &Vec<Rrset>) -> Rrsets {
        Rrsets {
            rrsets: rrsets.clone(),
        }
    }
}

#[allow(dead_code)]
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
               &self.id, &self.name, &self.type_id.clone().unwrap_or(StructType::None),
               &self.kind, &self.url,
               rrsets.join(", "), self.serial, self.edited_serial,
               &self.masters.join(", "), self.dnssec, &self.nsec3param, self.nsec3narrow,
               self.presigned.clone().unwrap_or(false), &self.soa_edit, &self.soa_edit_api,
               self.api_rectify,
               &self.zone.clone().unwrap_or(String::new()),
               &self.account.clone().unwrap_or(String::new()),
               &self.nameservers.clone().unwrap_or(Vec::new()).join(", "),
               &self.master_tsig_key_ids.join(", "),
               &self.slave_tsig_key_ids.join(", "))
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
               &self.master_tsig_key_ids.clone().unwrap_or(Vec::new()).join(", "),
               &self.slave_tsig_key_ids.clone().unwrap_or(Vec::new()).join(", "))
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

fn canonicalize_name(name: &String) -> String {
    if !name.ends_with(".") {
        format!("{}.", name)
    } else {
        name.clone()
    }
}