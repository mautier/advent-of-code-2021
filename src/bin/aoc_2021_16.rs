use advent_of_code::bits::Bitstream;

fn main() {
    for test_file in ["2021-12-16.sample_literal.txt", "2021-12-16.txt"] {
        println!(
            "------------------------- {} -------------------------",
            test_file
        );
        let input_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let input = std::fs::read_to_string(&input_path).unwrap();
        let input = input.trim();
        let msg = parse_puzzle_input(input);

        // Display the bitstring... or at least the beginning of it.
        let mut bitstring = to_bitstring(&msg.bytes);
        if bitstring.len() >= 200 {
            bitstring = format!("{}...", &bitstring[..200.min(bitstring.len())]);
        }
        println!("Parsed hex string input:\n{}\n{}", &input, bitstring,);

        let packet = parse_message(&msg);
        println!("Parsed top-level packet:\n{:?}", packet);

        let mut to_visit = vec![&packet];
        let mut sum_of_versions = 0u64;
        while let Some(packet) = to_visit.pop() {
            sum_of_versions += packet.version as u64;

            if let Payload::Op { packets, .. } = &packet.payload {
                to_visit.extend(packets);
            }
        }

        println!("Part 1: sum of packet versions: {}", sum_of_versions);
        println!("Part 2: result = {}", eval(&packet));
    }
}

struct Message {
    bytes: Vec<u8>,
}

impl Message {
    fn as_bits(&self) -> Bitstream<'_> {
        Bitstream::new(&self.bytes)
    }
}

#[derive(Clone, Debug)]
struct Packet {
    version: u8,
    payload: Payload,
}

#[derive(Clone, Debug)]
enum Payload {
    Literal(u64),
    Op { id: OpId, packets: Vec<Packet> },
}

#[derive(Clone, Debug)]
enum OpId {
    Sum,
    Product,
    Min,
    Max,
    Greater,
    Less,
    Equal,
}

fn parse_message(msg: &Message) -> Packet {
    let mut bits = msg.as_bits();
    let packet = parse_packet(&mut bits);

    // There may be some bits left at the end. They must all be zeroes.
    assert!(bits.num_remaining_bits() < 8);
    let remaining_data = bits
        .get_n_bits(bits.num_remaining_bits() as u8)
        .unwrap_or(0);
    assert_eq!(remaining_data, 0);

    packet
}

fn parse_packet(bits: &mut Bitstream<'_>) -> Packet {
    let version = bits.get_n_bits(3).unwrap() as u8;
    let type_id = bits.get_n_bits(3).unwrap() as u8;

    let payload = match type_id {
        4 => Payload::Literal(parse_literal(bits)),
        0..=3 | 5..=7 => {
            use OpId::*;

            Payload::Op {
                id: match type_id {
                    0 => Sum,
                    1 => Product,
                    2 => Min,
                    3 => Max,
                    5 => Greater,
                    6 => Less,
                    7 => Equal,
                    _ => unreachable!(),
                },
                packets: parse_op_packets(bits),
            }
        }
        _ => panic!("Invalid op type id: {}", type_id),
    };

    Packet { version, payload }
}

fn parse_literal(bits: &mut Bitstream<'_>) -> u64 {
    let mut lit = 0u64;
    loop {
        let has_more_groups = bits.get_n_bits(1).unwrap() == 1;
        lit = (lit << 4) | bits.get_n_bits(4).unwrap() as u64;
        if !has_more_groups {
            break;
        }
    }

    lit
}

fn parse_op_packets(bits: &mut Bitstream<'_>) -> Vec<Packet> {
    let length_type_id = bits.get_n_bits(1).unwrap();

    let mut subpackets = Vec::new();

    if length_type_id == 0 {
        let len_of_subpackets_in_bits = bits.get_n_bits(15).unwrap() as usize;
        let bits_in_stream = bits.num_remaining_bits();

        // How many trailing bits are not part of this op's subpackets?
        let num_not_our_business_bits = bits_in_stream - len_of_subpackets_in_bits;

        while bits.num_remaining_bits() > num_not_our_business_bits {
            subpackets.push(parse_packet(bits));
        }
        assert_eq!(bits.num_remaining_bits(), num_not_our_business_bits);
    } else {
        let num_subpackets = bits.get_n_bits(11).unwrap();
        for _ in 0..num_subpackets {
            subpackets.push(parse_packet(bits));
        }
    }

    subpackets
}

fn eval(packet: &Packet) -> u64 {
    match &packet.payload {
        Payload::Literal(x) => *x,
        Payload::Op { id, packets } => {
            let mut evaled_subs = packets.iter().map(eval);

            use OpId::*;
            match id {
                Sum => evaled_subs.sum::<u64>(),
                Product => evaled_subs.product(),
                Min => evaled_subs.min().unwrap(),
                Max => evaled_subs.max().unwrap(),
                Greater => {
                    let a = evaled_subs.next().unwrap();
                    let b = evaled_subs.next().unwrap();
                    (a > b) as u64
                }
                Less => {
                    let a = evaled_subs.next().unwrap();
                    let b = evaled_subs.next().unwrap();
                    (a < b) as u64
                }
                Equal => {
                    let a = evaled_subs.next().unwrap();
                    let b = evaled_subs.next().unwrap();
                    (a == b) as u64
                }
            }
        }
    }
}

fn parse_puzzle_input(txt: &str) -> Message {
    let mut bytes = Vec::new();

    for chunk in txt.as_bytes().chunks(2) {
        // `chunk` is of size 1 or 2.
        let mut b = binary_from_ascii_hex_char(chunk[0]) << 4;
        if let Some(rest) = chunk.get(1) {
            b |= binary_from_ascii_hex_char(*rest);
        }

        bytes.push(b);
    }
    Message { bytes }
}

fn binary_from_ascii_hex_char(hex_char_ascii: u8) -> u8 {
    match hex_char_ascii {
        b'0'..=b'9' => hex_char_ascii - b'0',
        b'a'..=b'f' => 10 + (hex_char_ascii - b'a'),
        b'A'..=b'F' => 10 + (hex_char_ascii - b'A'),
        _ => panic!("Unknown char: {}", hex_char_ascii),
    }
}

fn to_bitstring(array: &[u8]) -> String {
    use std::fmt::Write;

    let mut s = String::with_capacity(array.len() * 8);
    for b in array {
        write!(&mut s, "{:08b}", b).unwrap();
    }

    s
}
