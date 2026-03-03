#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlaveId(u32);

impl SlaveId {
    pub const MASTER_ID: u32 = 0;
    pub const FIRST_SLAVE_ID: u32 = SlaveId::MASTER_ID + 1;

    pub const fn first_slave_id() -> Self {
        SlaveId(Self::FIRST_SLAVE_ID)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for SlaveId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == Self::MASTER_ID {
            Err(())
        } else {
            Ok(SlaveId(value))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Slaves {
    n_slaves: u32,
}

impl Slaves {
    pub fn new(n_slaves: u32) -> Self {
        Self { n_slaves }
    }

    pub fn empty() -> Self {
        Self { n_slaves: 0 }
    }

    pub fn n_slaves(&self) -> u32 {
        self.n_slaves
    }

    pub fn range(&self) -> Option<core::ops::RangeInclusive<u32>> {
        if self.n_slaves > 0 {
            Some(SlaveId::FIRST_SLAVE_ID..=(SlaveId::FIRST_SLAVE_ID + self.n_slaves - 1))
        } else {
            None
        }
    }
}

impl IntoIterator for Slaves {
    type Item = u32;

    type IntoIter = core::iter::Flatten<core::option::IntoIter<core::ops::RangeInclusive<u32>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.range().into_iter().flatten()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LiketrainMode {
    Master,
    Slave { slave_id: SlaveId },
}

impl LiketrainMode {
    pub fn is_master(&self) -> bool {
        matches!(self, LiketrainMode::Master)
    }

    pub fn get_slave_id(&self) -> u32 {
        match self {
            LiketrainMode::Master => SlaveId::MASTER_ID,
            LiketrainMode::Slave { slave_id } => slave_id.as_u32(),
        }
    }
}
