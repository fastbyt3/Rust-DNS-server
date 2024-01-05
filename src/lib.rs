#[derive(Debug, Clone, Copy)]
pub enum QueryResponseIndicator {
    Query = 0,
    Response = 1,
}

impl QueryResponseIndicator {
    fn from_byte(byte: u8) -> Self {
        let bit_val = byte >> 7;
        match bit_val {
            0 => QueryResponseIndicator::Query,
            1 => QueryResponseIndicator::Response,
            _ => panic!("QR indicator not 0 / 1"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Query = 0,
    InverseQuery = 1,
    Status = 2,
}

impl OpCode {
    fn from_byte(byte: u8) -> Self {
        let val = (byte & 0b01111000) >> 3;
        match val {
            0 => Self::Query,
            1 => Self::InverseQuery,
            2 => Self::Status,
            3..=15 => panic!("Opcode value reserved for future use"),
            _ => panic!("Invalid opcode value"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

impl ResponseCode {
    fn from_byte(byte: u8) -> Self {
        let val = byte & 0b00001111;
        match val {
            0 => Self::NoError,
            1 => Self::FormatError,
            2 => Self::ServerFailure,
            3 => Self::NameError,
            4 => Self::NotImplemented,
            5 => Self::Refused,
            _ => panic!("Invalid ResponseCode"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub id: u16,
    pub qr: QueryResponseIndicator,
    pub opcode: OpCode,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub z: bool,
    pub rcode: ResponseCode,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl Header {
    pub fn from_bytes(buf: &[u8]) -> Self {
        Self {
            id: ((buf[0] as u16) << 8) | buf[1] as u16,
            qr: QueryResponseIndicator::from_byte(buf[2]),
            opcode: OpCode::from_byte(buf[2]),
            aa: ((buf[2] & 0b00000100) >> 2) > 0,
            tc: ((buf[2] & 0b00000010) >> 1) > 0,
            rd: (buf[2] & 0b00000001) > 0,
            ra: (buf[3] >> 7) > 0,
            z: ((buf[3] & 0b01110000) >> 4) > 0,
            rcode: ResponseCode::from_byte(buf[3]),
            qdcount: ((buf[4] as u16) << 8) | buf[5] as u16,
            ancount: ((buf[6] as u16) << 8) | buf[7] as u16,
            nscount: ((buf[8] as u16) << 8) | buf[9] as u16,
            arcount: ((buf[10] as u16) << 8) | buf[11] as u16,
        }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut resp = [0; 12];

        resp[0..=1].copy_from_slice(&self.id.to_be_bytes());

        resp[2] = (self.qr as u8) << 7
            | (self.opcode as u8) << 3
            | (self.aa as u8) << 2
            | (self.tc as u8) << 1
            | (self.rd as u8);

        resp[3] = (self.ra as u8) << 7 | (self.z as u8) << 4 | (self.rcode as u8);

        resp[4..=5].copy_from_slice(&self.qdcount.to_be_bytes());
        resp[6..=7].copy_from_slice(&self.ancount.to_be_bytes());
        resp[8..=9].copy_from_slice(&self.nscount.to_be_bytes());
        resp[10..=11].copy_from_slice(&self.arcount.to_be_bytes());

        resp
    }
}
