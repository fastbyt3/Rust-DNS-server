# DNS Server in Rust -> Notes

## Message format

5 sections:
1. Header
2. Question
3. Authority
4. additional space

## Header

```

                                    1  1  1  1  1  1
      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      ID                       |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    QDCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    ANCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    NSCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    ARCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
```

- Always **12 bytes** long
- Integers are encoded in Big-endian

| Field                             | Size (bits) | Description                                  |
| --------------------------------- | ----------- | -------------------------------------------- |
| Packet Identifier (ID)            | 16          | Resp must have same ID                       |
| Query/Response Indicator (QR)     | 1           | 0 -> Question / 1 -> Resp                    |
| OperationCode (OPCODE)            | 4           | kind of query                                |
| Authority Answer (AA)             | 1           | 1 -> resp server "owns" queried domain       |
| Truncation (TC)                   | 1           | 1 -> msg size > 512 bytes; 0 -> for UDP resp |
| Recursion Desired (RD)            | 1           | 1 -> recursively resolve query               |
| Recursion Available (RA)          | 1           | 1 -> recursion is available                  |
| Reserved (Z)                      | 3           | Used by DNSSEC queries                       |
| Response Code (RCODE)             | 4           | Status of Resp                               |
| Question Count (QDCOUNT)          | 16          | #Questions in Question section               |
| Answer record Count (ANCOUNT)     | 16          | #records in Answer section                   |
| Authority record Count (NSCOUNT)  | 16          | #records in Authority section                |
| Additional record Count (ARCOUNT) | 16          | #records in Additional section               |

> Ref: [DNS Protocol](https://github.com/EmilHernvall/dnsguide/blob/b52da3b32b27c81e5c6729ac14fe01fef8b1b593/chapter1.md)
> [RFC](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1)

- OPCODE:
	- 0: a standard query (QUERY)
	- 1: an inverse query (IQUERY)
	- 2: a server status request (STATUS)
	- 3-15: reserved for future use
 
 ## Question section

 1. Name -> sequence of "labels" (encoded)
 2. Type -> **2-byte int** type of record
 3. Class -> **2-byte int** usually set to 1
 
### Domain name encoding

```
                                    1  1  1  1  1  1
      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                                               |
    /                     QNAME                     /
    /                                               /
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                     QTYPE                     |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                     QCLASS                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
	```

- `<length><content>..\x00`
    - length: 1 Byte -> label len
    - `\x00` -> terminator

```
\x06google\x03com\x00 => google.com
- read 6 chars
- read 3 chars
- null => break
```


