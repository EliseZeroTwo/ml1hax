use crate::{
    mmio::{self, mmio_or, mmio_read, mmio_write},
    utils::usleep,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2CError {
    Timeout,
    InvalidSize,
    MissingAck,
}

pub fn init(i2c_base: usize) -> Result<(), I2CError> {
    mmio_write(i2c_base, mmio::i2c::I2C_CLK_DIVISOR_REGISTER, (5 << 16) | 1);
    mmio_write(
        i2c_base,
        mmio::i2c::I2C_BUS_CLEAR_CONFIG,
        (9 << 16) | (1 << 1) | 1,
    );
    start_transaction_and_wait_until_idle(i2c_base, None)?;

    let mut x = 0;
    loop {
        usleep(0x4e20);

        if mmio_read(i2c_base, mmio::i2c::I2C_INTERRUPT_STATUS_REGISTER) & (1 << 11) != 0 {
            break;
        }

        if x > 9 {
            return Err(I2CError::Timeout);
        }

        x += 1;
    }

    mmio_read(i2c_base, mmio::i2c::I2C_BUS_CLEAR_STATUS);
    mmio_write(
        i2c_base,
        mmio::i2c::I2C_INTERRUPT_STATUS_REGISTER,
        mmio_read(i2c_base, mmio::i2c::I2C_INTERRUPT_STATUS_REGISTER),
    );

    Ok(())
}

pub fn start_transaction_and_wait_until_idle(
    i2c_base: usize,
    timeout_us: Option<u32>,
) -> Result<(), I2CError> {
    let timeout_us = timeout_us.unwrap_or(0x14);

    mmio_write(i2c_base, mmio::i2c::I2C_CONFIG_LOAD, (1 << 2) | (1 << 0));

    for _ in 0..timeout_us {
        usleep(1);
        if mmio_read(i2c_base, mmio::i2c::I2C_CONFIG_LOAD) & 1 == 0 {
            return Ok(());
        }
    }

    Err(I2CError::Timeout)
}

pub fn send_packet(
    i2c_base: usize,
    device: u8,
    buffer: &[u8],
) -> Result<(), I2CError> {
    if buffer.len() > 4 || buffer.is_empty() {
        return Err(I2CError::InvalidSize);
    }

    let mut bytes = [0u8; 4];

    bytes[..buffer.len()].copy_from_slice(buffer);

    let data = u32::from_le_bytes(bytes);

    mmio_write(
        i2c_base,
        mmio::i2c::I2C_PRIMARY_CNFG,
        (1 << 11) | (2 << 12) | ((buffer.len() as u32 - 1) << 1),
    );
    mmio_write(
        i2c_base,
        mmio::i2c::I2C_CMD_ADDR0,
        (device as u32 & 0b0111_1111) << 1,
    );
    mmio_write(i2c_base, mmio::i2c::I2C_CMD_DATA1, data);
    start_transaction_and_wait_until_idle(i2c_base, None)?;

    mmio_or(i2c_base, mmio::i2c::I2C_PRIMARY_CNFG, 0x200);

    while mmio_read(i2c_base, mmio::i2c::I2C_STATUS) & (1 << 8) != 0 {}

    if mmio_read(i2c_base, mmio::i2c::I2C_STATUS) & 0xF != 0 {
        return Err(I2CError::MissingAck);
    }

    Ok(())
}

pub fn receive_packet(
    i2c_base: usize,
    device: u8,
    buffer: &mut [u8],
) -> Result<(), I2CError> {
    if buffer.len() > 4 || buffer.is_empty() {
        return Err(I2CError::InvalidSize);
    }

    mmio_write(
        i2c_base,
        mmio::i2c::I2C_PRIMARY_CNFG,
        (1 << 6) | (1 << 11) | (2 << 12) | ((buffer.len() as u32 - 1) << 1),
    );
    mmio_write(
        i2c_base,
        mmio::i2c::I2C_CMD_ADDR0,
        ((device as u32 & 0b0111_1111) << 1) | 1,
    );

    start_transaction_and_wait_until_idle(i2c_base, None)?;

    while mmio_read(i2c_base, mmio::i2c::I2C_STATUS) & (1 << 8) != 0 {}

    if mmio_read(i2c_base, mmio::i2c::I2C_STATUS) & 0xF != 0 {
        return Err(I2CError::MissingAck);
    }

    let data = mmio_read(i2c_base, mmio::i2c::I2C_CMD_DATA1);

    buffer.copy_from_slice(&data.to_le_bytes()[..buffer.len()]);

    Ok(())
}

pub fn send_typed_byte(
    i2c_base: usize,
    device: u8,
    r#type: u8,
    byte: u8,
) -> Result<(), I2CError> {
    send_packet(i2c_base, device, &[r#type, byte])
}

pub fn receive_typed_byte(
    i2c_base: usize,
    device: u8,
    r#type: u8,
) -> Result<u8, I2CError> {
    send_packet(i2c_base, device, &[r#type])?;

    let mut buffer = [0u8];
    receive_packet(i2c_base, device, &mut buffer)?;

    Ok(buffer[0])
}
