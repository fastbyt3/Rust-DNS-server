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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Query = 0,
    InverseQuery = 1,
    Status = 2,
    FutureUse,
}

impl OpCode {
    fn from_byte(byte: u8) -> Self {
        let val = (byte & 0b01111000) >> 3;
        match val {
            0 => Self::Query,
            1 => Self::InverseQuery,
            2 => Self::Status,
            3..=15 => Self::FutureUse,
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
        let mut x = Self {
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
        };
        x.rcode = if x.opcode == OpCode::Query {
            ResponseCode::NoError
        } else {
            ResponseCode::NotImplemented
        };
        x
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
    pub name: Label,
    pub qtype: u16,
    pub class: u16,
}

impl Question {
    pub fn from_bytes(buf: &[u8]) -> Self {
        let (name, bytes_read) = Label::decode(buf);
        let qtype = ((buf[bytes_read + 1] as u16) << 8) | buf[bytes_read + 2] as u16;
        let class = ((buf[bytes_read + 3] as u16) << 8) | buf[bytes_read + 4] as u16;

        Question { name, qtype, class }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut resp = self.name.encode();

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
    pub answers: Vec<Answer>,
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

        let answers = Vec::new();

        Message {
            header,
            questions,
            answers,
        }
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
        for a in &self.answers {
            let ab = a.to_bytes();
            for i in 0..ab.len() {
                resp[start + i] = ab[i];
            }
        }
        resp
    }

    pub fn prepare_response(&mut self) {
        let mut answers = Vec::new();
        // answers.push(Answer {
        //     name: String::from("codecrafters.io"),
        //     atype: AnswerType::A,
        //     class: 1,
        //     ttl: 60,
        //     rdlength: 4,
        //     rdata: [8, 8, 8, 8].to_vec(),
        // });
        // println!("{answers:?}");
        self.header.qr = QueryResponseIndicator::Response;
        self.header.qdcount = self.questions.len() as u16;
        for i in 0..self.header.qdcount {
            let q = self.questions.get(i as usize).unwrap();
            answers.push(Answer {
                name: q.name.clone(),
                atype: AnswerType::A,
                class: 1,
                ttl: 60,
                rdlength: 4,
                rdata: [8, 8, 8, 8].to_vec(),
            });
        }
        self.answers = answers;
        self.header.ancount = self.answers.len() as u16;
    }
}

// For this stage
// --------------
// Name: codecrafters.io
// Type: 1 <- AA
// Class: 1 <- IN record class
// TTL: 60 (or any val)
// RDLEN: 4
// RDATA: Any ip addr: \x08\x08\x08\x08 => 8.8.8.8
//
// Also update ANCOUNT in Header
#[derive(Debug, Clone, Copy)]
pub enum AnswerType {
    A = 1,
    NS = 2,
    MD = 3,
    MF = 4,
    CNAME = 5,
    SOA = 6,
    MB = 7,
    MG = 8,
    MR = 9,
    NULL = 10,
    WKS = 11,
    PTR = 12,
    HINFO = 13,
    MINFO = 14,
    MX = 15,
    TXT = 16,
}

impl AnswerType {
    fn get_val(&self) -> u16 {
        self.clone() as u16
    }
}

#[derive(Debug)]
pub struct Answer {
    pub name: Label,
    pub atype: AnswerType,
    pub class: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl Answer {
    fn to_bytes(&self) -> Vec<u8> {
        let mut answer_bytes = self.name.encode();
        answer_bytes.extend_from_slice(&self.atype.get_val().to_be_bytes());
        answer_bytes.extend_from_slice(&self.class.to_be_bytes());
        answer_bytes.extend_from_slice(&self.ttl.to_be_bytes());
        answer_bytes.extend_from_slice(&self.rdlength.to_be_bytes());
        answer_bytes.extend_from_slice(&self.rdata);
        answer_bytes
    }
}

#[derive(Debug, Clone)]
pub struct Label(String);

impl Label {
    fn decode(buf: &[u8]) -> (Label, usize) {
        let mut bytes_to_read: usize;
        let mut name = String::new();
        let mut i: usize = 0;
        loop {
            bytes_to_read = buf[i] as usize;
            // PTR check
            // first 2 bits == 1
            if bytes_to_read & 0b11000000 == 0b11000000 {
                name.push('.');
                let offset =
                    (((bytes_to_read & 0b00111111) as u16) << 8 | buf[i + 1] as u16) as usize;
                let (repeated_label, _) = Label::decode(&buf[offset..]);
                repeated_label.0.chars().for_each(|c| name.push(c));
                println!("Label got using offset: {}", name);
                i += 1;
                break;
            }
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
        (Label(name), i)
    }

    fn encode(&self) -> Vec<u8> {
        let mut enc = Vec::new();
        self.0.split('.').for_each(|s| {
            enc.push(s.len() as u8);
            s.chars().for_each(|c| enc.push(c as u8));
        });
        enc.push(0);
        enc
    }
}
