
use core::arch::asm;

pub fn test_red(x: f32, y: f32) -> f32 {
    let f: fn(f32,f32) -> f32 = std::hint::black_box(libm::atan2f);
    time(x,y,f)
}
pub fn test_green(x: f32, y: f32) -> f32 {
    let f: fn(f32,f32) -> f32 = std::hint::black_box(|x,y| x.atan2(y));
    time(x,y,f)
}

pub fn test_blue(x: f32, y: f32) -> f32 {
    test_green(x,y) / test_red(x,y)
}



pub fn test_red_i32(x: i32, y: i32) -> f32 {
    test_red(Total32(x).into(), Total32(y).into())
}

pub fn test_green_i32(x: i32, y: i32) -> f32 {
    test_green(Total32(x).into(), Total32(y).into())
}

pub fn test_blue_i32(x: i32, y: i32) -> f32 {
    test_blue(Total32(x).into(), Total32(y).into())
}

#[no_mangle]
pub fn int_fmt(x: i32) -> String {
    format!("{}", Total32(x))
}

#[no_mangle]
pub fn test_batched(x: &[i32], y: &[i32], r: &mut [f32], id: u32) {
    match id {
        0 => test_batched_gen(x,y,r,test_red_i32),
        1 => test_batched_gen(x,y,r,test_green_i32),
        2 => test_batched_gen(x,y,r,test_blue_i32),
        _ => return,
    }
}

pub fn test_batched_gen(x: &[i32], y: &[i32], r: &mut [f32], f: impl Fn(i32, i32) -> f32) {
    let n = x.len();
    assert_eq!(n, y.len());
    assert_eq!(n, r.len());

    for k in 0..n {
        r[k] = f(x[k], y[k]);
    }
}


// repetitions to time
const N: usize = 1 << 5;
const S: f32 = 1.0 / 512.0 / N as f32;
fn time(x: f32, y: f32, f: impl Fn(f32,f32) -> f32) -> f32 {
    unsafe {
        let mut xs = [x; N];
        let mut ys = [y; N];
        let mut zs = [0.0; N];
        let t0: u32;
        let t1: u32;
        asm!(
            "/* {} {} */",
            "mfence",
            "lfence",
            "rdtsc",
            in(reg) &mut xs,
            in(reg) &mut ys,
            out("eax") t0,
            out("edx") _,
            options(preserves_flags, nostack),
        );
        for i in 0..N {
            zs[i] = f(xs[i],ys[i]);
        }
        asm!(
            "/* {} */",
            "lfence",
            "rdtsc",
            "lfence",
            in(reg) &zs,
            out("eax") t1,
            out("edx") _,
            options(preserves_flags, nostack),
        );
        (t1.wrapping_sub(t0) as f32) * S
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
struct Total32(pub i32);

impl std::fmt::Display for Total32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0;
        write!(f, "{}", if s < 0 { '-' } else { '+' })?;
        let s = s ^ (s >> 31);

        let exp = s >> 23;
        let sig = s & ((1 << 23) - 1);

        if s == 0 {
            write!(f, "0x0.000000")?;
        } else if exp == 255 {
            if sig == 0 {
                write!(f, "Inf")?;
            } else {
                let quiet = sig >= (1 << 22);
                write!(f, "{}NaN(0x{:06x})",
                    if quiet { 'q' } else { 's' },
                    s & ((1 << 22) - 1)
                )?;
            }
        } else {
            write!(f, "0x{}.{:06x}p{}",
                exp.min(1),
                2 * sig,
                (exp - 127).max(-126)
            )?;
        }
        Ok(())
    }
}
impl From<f32> for Total32 {
    fn from(value: f32) -> Self {
        let i = value.to_bits() as i32;
        Self(i ^ (i >> 31) ^ (i & (1 << 31)))
    }
}
impl From<Total32> for f32 {
    fn from(Total32(i): Total32) -> Self {
        let u = i ^ (i >> 31) ^ (i & (1 << 31));
        f32::from_bits(u as u32)
    }
}