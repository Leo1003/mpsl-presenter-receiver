use crate::hid_report::MouseButtons;
use core::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Commands {
    AbsMove(u16, u16),
    RelMove(i16, i16),
    MouseDown(MouseButtons),
    MouseUp(MouseButtons),
    KeyDown(u8),
    KeyUp(u8),
    Wheel(i8),
}

impl FromStr for Commands {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut argv = s.split_ascii_whitespace();
        if let Some(cmd) = argv.next() {
            match cmd {
                "ma" => parse_ma(argv),
                "mr" => parse_mr(argv),
                "md" => parse_md(argv),
                "mu" => parse_mu(argv),
                "kd" => parse_kd(argv),
                "ku" => parse_ku(argv),
                "wh" => parse_wh(argv),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

fn parse_ma<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_x = iter.next();
    let arg_y = iter.next();

    if let Some((arg_x, arg_y)) = arg_x.zip(arg_y) {
        let x: u16 = arg_x.parse().map_err(|_| ())?;
        let y: u16 = arg_y.parse().map_err(|_| ())?;

        Ok(Commands::AbsMove(x, y))
    } else {
        Err(())
    }
}

fn parse_mr<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_x = iter.next();
    let arg_y = iter.next();

    if let Some((arg_x, arg_y)) = arg_x.zip(arg_y) {
        let x: i16 = arg_x.parse().map_err(|_| ())?;
        let y: i16 = arg_y.parse().map_err(|_| ())?;

        Ok(Commands::RelMove(x, y))
    } else {
        Err(())
    }
}

fn parse_md<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_btn = iter.next();

    if let Some(arg_btn) = arg_btn {
        let btn_bits: u8 = arg_btn.parse().map_err(|_| ())?;
        let btn = MouseButtons::from_bits(btn_bits).ok_or(())?;

        Ok(Commands::MouseDown(btn))
    } else {
        Err(())
    }
}

fn parse_mu<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_btn = iter.next();

    if let Some(arg_btn) = arg_btn {
        let btn_bits: u8 = arg_btn.parse().map_err(|_| ())?;
        let btn = MouseButtons::from_bits(btn_bits).ok_or(())?;

        Ok(Commands::MouseUp(btn))
    } else {
        Err(())
    }
}

fn parse_kd<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_key = iter.next();

    if let Some(arg_key) = arg_key {
        let keycode: u8 = arg_key.parse().map_err(|_| ())?;

        Ok(Commands::KeyDown(keycode))
    } else {
        Err(())
    }
}

fn parse_ku<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_key = iter.next();

    if let Some(arg_key) = arg_key {
        let keycode: u8 = arg_key.parse().map_err(|_| ())?;

        Ok(Commands::KeyUp(keycode))
    } else {
        Err(())
    }
}

fn parse_wh<'a, I>(mut iter: I) -> Result<Commands, ()>
where
    I: Iterator<Item = &'a str>,
{
    let arg_wheel = iter.next();

    if let Some(arg_wheel) = arg_wheel {
        let wheel: i8 = arg_wheel.parse().map_err(|_| ())?;

        Ok(Commands::Wheel(wheel))
    } else {
        Err(())
    }
}
