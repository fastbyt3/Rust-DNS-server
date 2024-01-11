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

#[derive(Debug)]
pub struct Question {
    pub name: String,
    pub qtype: u16,
    pub class: u16,
}

impl Question {
    pub fn from_bytes(buf: &[u8]) -> Self {
        let mut bytes_to_read: usize;
        let mut name = String::new();
        let mut i: usize = 0;
        loop {
            bytes_to_read = buf[i] as usize;
            // end of q
            if bytes_to_read == 0 {
                break;
            }
            // start of str -> dont add '.'
            // else for each end of part add '.'
            if i != 0 {
                name.push('.');
            }
            // start from next char till end
            (1..bytes_to_read + 1).for_each(|x| {
                name.push(buf[i + x] as char);
            });
            i += bytes_to_read + 1;
        }

        let qtype = ((buf[i + 1] as u16) << 8) | buf[i + 2] as u16;
        let class = ((buf[i + 3] as u16) << 8) | buf[i + 4] as u16;

        Question { name, qtype, class }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut resp = Vec::new();

        self.name.split('.').for_each(|x| {
            resp.push(x.len() as u8);
            x.chars().for_each(|c| {
                resp.push(c as u8);
            });
        });

        resp.push(0);

        resp.push((self.qtype >> 8) as u8);
        resp.push(self.qtype as u8);

        resp.push((self.class >> 8) as u8);
        resp.push(self.class as u8);

        resp
    }
}

#[derive(Debug)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
}

impl Message {
    pub fn from_bytes(req: [u8; 512]) -> Message {
        let header = Header::from_bytes(&req[..12]);
        let mut questions = Vec::new();
        let mut start = 12;

        for _ in 0..header.qdcount {
            let mut end_of_q = start;
            while req[end_of_q] != 0 {
                end_of_q += 1;
            }
            end_of_q += 1;
            end_of_q += 4;
            let q = &req[start..end_of_q];
            questions.push(Question::from_bytes(&q));
            start += end_of_q;
        }
        println!("Questions: {questions:?}");

        Message { header, questions }
    }

    pub fn to_bytes(&self) -> [u8; 512] {
        let mut resp = [0 as u8; 512];
        resp[..12].copy_from_slice(&self.header.to_bytes());
        let mut start = 12;
        for q in &self.questions {
            let qbytes = q.to_bytes();
            for i in 0..qbytes.len() {
                resp[start + i] = qbytes[i];
            }
            // resp[start..qbytes.len()].copy_from_slice(&qbytes);
            start += qbytes.len();
        }
        resp
    }

    pub fn format_as_response_message(&mut self) {
        self.header.qr = QueryResponseIndicator::Response;
        self.header.qdcount = self.questions.len() as u16;
    }
}
