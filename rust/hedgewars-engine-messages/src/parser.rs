use nom::{*, Err::Error};
use std::str;

use super::messages::{*, EngineMessage::*, UnsyncedEngineMessage::*, SyncedEngineMessage::*, ConfigEngineMessage::*, KeystrokeAction::*};


macro_rules! eof_slice (
  ($i:expr,) => (
    {
      if ($i).input_len() == 0 {
        Ok(($i, $i))
      } else {
        Err(Error(error_position!($i, ErrorKind::Eof::<u32>)))
      }
    }
  );
);

named!(unrecognized_message<&[u8], EngineMessage>,
    do_parse!(rest >> (Unknown))
);

named!(string_tail<&[u8], String>, map!(map_res!(rest, str::from_utf8), String::from));

named!(length_without_timestamp<&[u8], usize>,
    map_opt!(rest_len, |l| if l > 2 { Some(l - 2) } else { None } )
);

named!(synced_message<&[u8], SyncedEngineMessage>, alt!(
      do_parse!(tag!("L") >> (Left(Press)))
      | do_parse!(tag!("l") >> ( Left(Release) ))
      | do_parse!(tag!("R") >> ( Right(Press) ))
      | do_parse!(tag!("r") >> ( Right(Release) ))
      | do_parse!(tag!("U") >> ( Up(Press) ))
      | do_parse!(tag!("u") >> ( Up(Release) ))
      | do_parse!(tag!("D") >> ( Down(Press) ))
      | do_parse!(tag!("d") >> ( Down(Release) ))
      | do_parse!(tag!("Z") >> ( Precise(Press) ))
      | do_parse!(tag!("z") >> ( Precise(Release) ))
      | do_parse!(tag!("A") >> ( Attack(Press) ))
      | do_parse!(tag!("a") >> ( Attack(Release) ))
      | do_parse!(tag!("N") >> ( NextTurn ))
      | do_parse!(tag!("j") >> ( LongJump ))
      | do_parse!(tag!("J") >> ( HighJump ))
      | do_parse!(tag!("S") >> ( Switch ))
      | do_parse!(tag!(",") >> ( Skip ))
      | do_parse!(tag!("1") >> ( Timer(1) ))
      | do_parse!(tag!("2") >> ( Timer(2) ))
      | do_parse!(tag!("3") >> ( Timer(3) ))
      | do_parse!(tag!("4") >> ( Timer(4) ))
      | do_parse!(tag!("5") >> ( Timer(5) ))
      | do_parse!(tag!("p") >> x: be_i24 >> y: be_i24 >> ( Put(x, y) ))
      | do_parse!(tag!("P") >> x: be_i24 >> y: be_i24 >> ( CursorMove(x, y) ))
      | do_parse!(tag!("f") >> s: string_tail >> ( SyncedEngineMessage::TeamControlLost(s) ))
      | do_parse!(tag!("g") >> s: string_tail >> ( SyncedEngineMessage::TeamControlGained(s) ))
      /*
    Slot(u8),
    SetWeapon(u8),
          */
));

named!(unsynced_message<&[u8], UnsyncedEngineMessage>, alt!(
      do_parse!(tag!("?") >> (Ping))
    | do_parse!(tag!("!") >> (Ping))
    | do_parse!(tag!("esay ") >> s: string_tail >> (Say(s)))
));

named!(config_message<&[u8], ConfigEngineMessage>, alt!(
    do_parse!(tag!("C") >> (ConfigRequest))
));

named!(timestamped_message<&[u8], (SyncedEngineMessage, u16)>,
    do_parse!(msg: length_value!(length_without_timestamp, terminated!(synced_message, eof_slice!()))
        >> timestamp: be_u16
        >> ((msg, timestamp))
    )
);

named!(unwrapped_message<&[u8], EngineMessage>,
    alt!(
        map!(timestamped_message, |(m, t)| Synced(m, t as u32))
        | do_parse!(tag!("#") >> (Synced(TimeWrap, 65535)))
        | map!(unsynced_message, |m| Unsynced(m))
        | map!(config_message, |m| Config(m))
        | unrecognized_message
));



named!(length_specifier<&[u8], u16>, alt!(
    verify!(map!(take!(1), |a : &[u8]| a[0] as u16), |l| l < 64)
    | map!(take!(2), |a| (a[0] as u16 - 64) * 256 + a[1] as u16 + 64)
    )
);

named!(empty_message<&[u8], EngineMessage>,
    do_parse!(tag!("\0") >> (Empty))
);

named!(non_empty_message<&[u8], EngineMessage>,
    length_value!(length_specifier, terminated!(unwrapped_message, eof_slice!())));

named!(message<&[u8], EngineMessage>, alt!(
      empty_message
    | non_empty_message
    )
);

named!(pub extract_messages<&[u8], Vec<EngineMessage> >, many0!(complete!(message)));

#[test]
fn parse_length() {
    assert_eq!(length_specifier(b"\x01"), Ok((&b""[..], 1)));
    assert_eq!(length_specifier(b"\x00"), Ok((&b""[..], 0)));
    assert_eq!(length_specifier(b"\x3f"), Ok((&b""[..], 63)));
    assert_eq!(length_specifier(b"\x40\x00"), Ok((&b""[..], 64)));
    assert_eq!(length_specifier(b"\xff\xff"), Ok((&b""[..], 49215)));
}

#[test]
fn parse_synced_messages() {
    assert_eq!(message(b"\x03L\x01\x02"), Ok((&b""[..], Synced(Left(Press), 258))));
    assert_eq!(message(b"\x01#"), Ok((&b""[..], Synced(TimeWrap, 65535))));
}

#[test]
fn parse_unsynced_messages() {
    assert_eq!(message(b"\x0aesay hello"), Ok((&b""[..], Unsynced(Say(String::from("hello"))))));
}

#[test]
fn parse_incorrect_messages() {
    assert_eq!(message(b"\x00"), Ok((&b""[..], Empty)));
    assert_eq!(message(b"\x01\x00"), Ok((&b""[..], Unknown)));

    // garbage after correct message
    assert_eq!(message(b"\x04La\x01\x02"), Ok((&b""[..], Unknown)));
}

#[test]
fn parse_config_messages() {
    assert_eq!(
        message(b"\x01C"),
        Ok((
            &b""[..],
            Config(ConfigRequest)
        ))
    );
}
#[test]
fn parse_test_general() {
    assert_eq!(string_tail(b"abc"), Ok((&b""[..], String::from("abc"))));
}
