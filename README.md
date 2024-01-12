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

## Answer section

- contains a list of RRs (Resource records)

```
                                    1  1  1  1  1  1
      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                                               |
    /                                               /
    /                      NAME                     /
    |                                               |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      TYPE                     |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                     CLASS                     |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      TTL                      |
    |                                               |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                   RDLENGTH                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
    /                     RDATA                     /
    /                                               /
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
```

| Field          | Type       | Description                   |
| -------------- | ---------- | ----------------------------- |
| Name           | label seq  | domain name encoded           |
| Type           | 2byte Int  | A record = 1; ..              |
| Class          | 2byte Int  | usually set to 1              |
| TTL            | 4Byte Int  | cache before requery period   |
| Len (RDLENGTH) | 2 Byte Int | Len of RDATA field            |
| RDATA          | varaible   | data specific to record types |

### types

| TYPE  | value and meaning                          |
| ----- | ------------------------------------------ |
| A     | 1 a host address                           |
| NS    | 2 an authoritative name server             |
| MD    | 3 a mail destination (Obsolete - use MX)   |
| MF    | 4 a mail forwarder (Obsolete - use MX)     |
| CNAME | 5 the canonical name for an alias          |
| SOA   | 6 marks the start of a zone of authority   |
| MB    | 7 a mailbox domain name (EXPERIMENTAL)     |
| MG    | 8 a mail group member (EXPERIMENTAL)       |
| MR    | 9 a mail rename domain name (EXPERIMENTAL) |
| NULL  | 10 a null RR (EXPERIMENTAL)                |
| WKS   | 11 a well known service description        |
| PTR   | 12 a domain name pointer                   |
| HINFO | 13 host information                        |
| MINFO | 14 mailbox or mail list information        |
| MX    | 15 mail exchange                           |
| TXT   | 16 text strings                            |

## Parsing compressed Packet

- eliminates the repetition of domain names in a message
- entire domain name or a list of labels at end of a domain name is replaced with pointer to prev occurance
- ptr -> 2 octet sequence

```
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
| 1  1|                OFFSET                   |
+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
```

- First 2 bits are 1s => distinguish ptrs from labels (both bits are 0)
- OFFSET => from start of msg
    - offset = 0 => first byte of ID field

```
F.ISI.ARPA,
FOO.F.ISI.ARPA,
ARPA

       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    20 |           1           |           F           |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    22 |           3           |           I           |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    24 |           S           |           I           |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    26 |           4           |           A           |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    28 |           R           |           P           |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    30 |           A           |           0           | => End of F.ISI.ARPA
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    40 |           3           |           F           |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    42 |           O           |           O           | => FOO part of FOO.F.ISI.ARPA
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    44 | 1  1|                20                       | => PTR offset = 20
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    64 | 1  1|                26                       | => for ARPA: PTR offset = 26
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    92 |           0           |                       |
       +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
```
