use std::net::Ipv4Addr;

use crate::{domain::Domain, message::Consume};

/// See <https://en.wikipedia.org/wiki/List_of_DNS_record_types>
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResourceRecordType {
    A { ipv4: Ipv4Addr },
    AAAA,
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

// Converts from `(global buffer, ptr to rtype)`
impl TryFrom<(&[u8], usize)> for ResourceRecordType {
    type Error = ();

    fn try_from(from: (&[u8], usize)) -> Result<Self, Self::Error> {
        let rtype = u16::from_be_bytes(from.0[from.1..from.1 + 2].try_into().unwrap());
        let rdata_starts_at = from.1 + 10;

        Ok(match rtype {
            1 => Self::A {
                ipv4: Ipv4Addr::new(
                    from.0[rdata_starts_at],
                    from.0[rdata_starts_at + 1],
                    from.0[rdata_starts_at + 2],
                    from.0[rdata_starts_at + 3],
                ),
            },
            28 => Self::AAAA,
            18 => Self::AFSDB,
            42 => Self::APL,
            257 => Self::CAA,
            60 => Self::CDNSKEY,
            59 => Self::CDS,
            37 => Self::CERT,
            5 => Self::CNAME {
                alias: Domain::read(from.0, rdata_starts_at)?.0,
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
                ns: Domain::read(from.0, rdata_starts_at)?.0,
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
                let mut ptr = rdata_starts_at;
                let (ns, bytes_read) = Domain::read(from.0, ptr)?;
                ptr += bytes_read;

                let (mail, _bytes_read) = Domain::read(from.0, ptr)?;

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
        })
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
