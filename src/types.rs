use std::{fmt, ops, str::FromStr};

use rand::Rng;

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub(crate) struct Id(pub(crate) [u8; 20]);

impl Id {
    pub(crate) const SIZE: usize = 20;

    pub(crate) fn random<R: Rng>(rng: &mut R) -> Self {
        let mut id = [0; 20];
        rng.fill(&mut id);
        Self(id)
    }

    pub(crate) fn leading_zeros(&self) -> u32 {
        self.0
            .iter()
            .copied()
            .enumerate()
            .find_map(|(idx, byte)| (byte != 0).then(|| idx as u32 * 8 + byte.leading_zeros()))
            .unwrap_or(Self::SIZE as u32 * 8)
    }
}

impl ops::BitXor for Id {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut idx = 0;
        Self([(); 20].map(|_| {
            let byte = self.0[idx] ^ rhs.0[idx];
            idx += 1;
            byte
        }))
    }
}

impl FromStr for Id {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = [0; 20];
        hex::decode_to_slice(s, &mut id)?;
        Ok(Self(id))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; 40];
        hex::encode_to_slice(self.0, &mut buf).map_err(|_| fmt::Error)?;
        f.write_str(std::str::from_utf8(&buf).map_err(|_| fmt::Error)?)
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

pub(crate) struct ReverseDistance<C> {
    pub(crate) distance: Id,
    pub(crate) contact: C,
}

impl<C> Ord for ReverseDistance<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl<C> PartialOrd for ReverseDistance<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.distance.cmp(&self.distance))
    }
}

impl<C> PartialEq for ReverseDistance<C> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl<C> Eq for ReverseDistance<C> {}
