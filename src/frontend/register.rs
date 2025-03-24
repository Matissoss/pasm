//  rasmx86_64  -   register.rs
//  ---------------------------
//  made by matissoss
//  licensed under MPL 2.0

use std::str::FromStr;

#[allow(unused)]
#[derive(Debug)]
#[derive(PartialEq, Default)]
pub enum Register{
    Bit64(Register64bit),
    Bit32(Register32bit),
    Bit16(Register16bit),
    Bit8 (Register8bit ),
    #[default]
    Unknown
}


#[allow(unused)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Register32bit{
    EAX,
    EBX,
    ECX,
    EDX,
    EBP,
    ESP,
    ESI,
    EDI
}

#[allow(unused)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Register16bit{
    AX,
    BX,
    CX,
    DX
}

#[allow(unused)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Register8bit{
    AL,
    AH,
    BL,
    BH,
    CL,
    CH,
    DL,
    DH
}

#[allow(unused)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Register64bit{
    RAX,
    RBX,
    RCX,
    RDX,
    RBP,
    RSP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15
}

impl FromStr for Register{
    type Err = ();
    fn from_str(str: &str) -> Result<Self, <Self as FromStr>::Err>{
        let byte_str = str.as_bytes();

        if byte_str.len() >= 2 {
            if byte_str[0] == 'r' as u8{
                if let Some(end_byte) = byte_str.get(2){
                    match *end_byte as char {
                        'x' => {
                            match byte_str[1] as char{
                                'a' => return Ok(Self::Bit64(Register64bit::RAX)),
                                'b' => return Ok(Self::Bit64(Register64bit::RBX)),
                                'c' => return Ok(Self::Bit64(Register64bit::RCX)),
                                'd' => return Ok(Self::Bit64(Register64bit::RDX)),
                                _   => return Err(()) 
                            }
                        }
                        'i' => {
                            match byte_str[1] as char{
                                's' => return Ok(Self::Bit64(Register64bit::RSI)),
                                'd' => return Ok(Self::Bit64(Register64bit::RDI)),
                                _   => return Err(()) 
                            }
                        }
                        'p' => {
                            match byte_str[1] as char {
                                'b' => return Ok(Self::Bit64(Register64bit::RBP)),
                                's' => return Ok(Self::Bit64(Register64bit::RSP)),
                                _   => return Err(())
                            }
                        }
                        '0' => return Ok(Self::Bit64(Register64bit::R10)),
                        '1' => return Ok(Self::Bit64(Register64bit::R11)),
                        '2' => return Ok(Self::Bit64(Register64bit::R12)),
                        '3' => return Ok(Self::Bit64(Register64bit::R13)),
                        '4' => return Ok(Self::Bit64(Register64bit::R14)),
                        '5' => return Ok(Self::Bit64(Register64bit::R15)),
                        _ => return Err(())
                    }
                }
            }
            else if byte_str[0] == 'e' as u8{
                if let Some(end_byte) = byte_str.get(2){
                    match *end_byte as char {
                        'x' => {
                            match byte_str[1] as char{
                                'a' => return Ok(Self::Bit32(Register32bit::EAX)),
                                'b' => return Ok(Self::Bit32(Register32bit::EBX)),
                                'c' => return Ok(Self::Bit32(Register32bit::ECX)),
                                'd' => return Ok(Self::Bit32(Register32bit::EDX)),
                                _   => return Err(()) 
                            }
                        }
                        'i' => {
                            match byte_str[1] as char{
                                's' => return Ok(Self::Bit32(Register32bit::ESI)),
                                'd' => return Ok(Self::Bit32(Register32bit::EDI)),
                                _   => return Err(()),
                            }
                        }
                        'p' => {
                            match byte_str[1] as char {
                                'b' => return Ok(Self::Bit32(Register32bit::EBP)),
                                's' => return Ok(Self::Bit32(Register32bit::ESP)),
                                _   => return Err(())
                            }
                        }
                        _ => return Err(())
                    }
                }
            }
            else if byte_str[1] == 'x' as u8{
                match byte_str[0] as char{
                    'a' => return Ok(Self::Bit16(Register16bit::AX)),
                    'b' => return Ok(Self::Bit16(Register16bit::BX)),
                    'c' => return Ok(Self::Bit16(Register16bit::CX)),
                    'd' => return Ok(Self::Bit16(Register16bit::DX)),
                    _ => {}
                }
            }
            else{
                if byte_str[1] == 'h' as u8{
                    match byte_str[0] as char{
                        'a' => return Ok(Self::Bit8(Register8bit::AH)),
                        'b' => return Ok(Self::Bit8(Register8bit::BH)),
                        'c' => return Ok(Self::Bit8(Register8bit::CH)),
                        'd' => return Ok(Self::Bit8(Register8bit::DH)),
                        _ => {}
                    }
                }
                else if byte_str[1] == 'l' as u8{
                    match byte_str[0] as char{
                        'a' => return Ok(Self::Bit8(Register8bit::AL)),
                        'b' => return Ok(Self::Bit8(Register8bit::BL)),
                        'c' => return Ok(Self::Bit8(Register8bit::CL)),
                        'd' => return Ok(Self::Bit8(Register8bit::DL)),
                        _ => {}
                    }
                }
            }
        }
        Err(())
    }
}
