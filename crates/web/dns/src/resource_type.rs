use std::{io::Read, net};

use sl_std::read::ReadExt;

use crate::{domain::Domain, reader::Reader, DNSError};

/// See <https://en.wikipedia.org/wiki/List_of_DNS_record_types>
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceRecord {
    A { ipv4: net::Ipv4Addr },
    AAAA { ipv6: net::Ipv6Addr },
    AFSDB,
    APL,
    CAA,
    CDNSKEY,
    CDS,
    CERT,
    CNAME { alias: Domain },
    CSYNC,
    DHCID,
    DLV,
    DNAME,
    DNSKEY,
    DS,
    EUI48,
    EUI64,
    HINFO,
    HIP,
    HTTPS,
    IPSECKEY,
    KEY,
    KX,
    LOC,
    MX,
    NAPTR,
    NS { ns: Domain },
    NSEC,
    NSEC3,
    NSEC3PARAM,
    OPENPGPKEY,
    PTR,
    RRSIG,
    RP,
    SIG,
    SMIMEA,
    SOA { _ns: Domain, _mail: Domain },
    SRV,
    SSHFP,
    SVCB,
    TA,
    TKEY,
    TLSA,
    TSIG,
    TXT,
    URI,
    ZONEMD,
    UNKNOWN,
}

impl ResourceRecord {
    pub fn read_from(reader: &mut Reader, rtype: u16) -> Result<Self, DNSError> {
        let record = match rtype {
            1 => {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;

                Self::A {
                    ipv4: net::Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3]),
                }
            },
            28 => Self::AAAA {
                ipv6: net::Ipv6Addr::new(
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                    reader.read_be_u16()?,
                ),
            },
            18 => Self::AFSDB,
            42 => Self::APL,
            257 => Self::CAA,
            60 => Self::CDNSKEY,
            59 => Self::CDS,
            37 => Self::CERT,
            5 => Self::CNAME {
                alias: Domain::read_from(reader)?,
            },
            62 => Self::CSYNC,
            49 => Self::DHCID,
            32769 => Self::DLV,
            39 => Self::DNAME,
            48 => Self::DNSKEY,
            43 => Self::DS,
            108 => Self::EUI48,
            109 => Self::EUI64,
            13 => Self::HINFO,
            55 => Self::HIP,
            65 => Self::HTTPS,
            45 => Self::IPSECKEY,
            25 => Self::KEY,
            36 => Self::KX,
            29 => Self::LOC,
            15 => Self::MX,
            35 => Self::NAPTR,
            2 => Self::NS {
                ns: Domain::read_from(reader)?,
            },
            47 => Self::NSEC,
            50 => Self::NSEC3,
            51 => Self::NSEC3PARAM,
            61 => Self::OPENPGPKEY,
            12 => Self::PTR,
            46 => Self::RRSIG,
            17 => Self::RP,
            24 => Self::SIG,
            53 => Self::SMIMEA,
            6 => {
                let ns = Domain::read_from(reader)?;
                let mail = Domain::read_from(reader)?;

                Self::SOA {
                    _ns: ns,
                    _mail: mail,
                    // TODO missing fields
                }
            },
            33 => Self::SRV,
            44 => Self::SSHFP,
            64 => Self::SVCB,
            32768 => Self::TA,
            249 => Self::TKEY,
            52 => Self::TLSA,
            250 => Self::TSIG,
            16 => Self::TXT,
            256 => Self::URI,
            63 => Self::ZONEMD,
            _ => Self::UNKNOWN,
        };

        Ok(record)
    }
}

/// [Specification](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.4)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceRecordClass {
    /// The Internet
    IN,
    /// the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CS,
    CH,
    /// Hesiod [Dyer 87]
    HS,
    UNKNOWN,
}

impl From<u16> for ResourceRecordClass {
    fn from(from: u16) -> Self {
        match from {
            1 => Self::IN,
            2 => Self::CS,
            3 => Self::CH,
            4 => Self::HS,
            _ => Self::UNKNOWN,
        }
    }
}
