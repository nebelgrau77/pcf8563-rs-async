//! # Alarm
//! All alarm-related functions will be defined here
//!
//! As it is now, setting an alarm component (minutes, hours, day, weekday) enables alarm for this component
//! TO DO: Keep the enabled/disabled bit when setting the alarm components (minutes, hours, day, weekday)

use crate::Weekday;

use super::{
    decode_bcd, encode_bcd, I2c, BitFlags, Control, Error, Register, DEVICE_ADDRESS, PCF8563
};

/// Alarm settings: minutes, hours, day, weekday. All of them are optional.
pub struct AlarmSettings {
    /// Alarm set to minutes
    pub minutes: Option<u8>,
    /// Alarm set to hours
    pub hours: Option<u8>,
    /// Alarm set to day (e.g. the 17th)
    pub day: Option<u8>,
    /// Alarm set to specific weekday
    pub weekday: Option<Weekday>,
}

impl AlarmSettings {
    /// create new alarm settings
    pub fn new() -> Self {
        AlarmSettings { 
            minutes: None, 
            hours: None, 
            day: None, 
            weekday: None,
        }
    }

    /// Use minutes value
    pub fn with_minutes(mut self, minutes: u8) -> Self {
        self.minutes = Some(minutes);
        self
    }

    /// Use hours value
    pub fn with_hours(mut self, hours: u8) -> Self {
        self.hours = Some(hours);
        self
    }

    /// Use day value
    pub fn with_day(mut self, day: u8) -> Self {
        self.day = Some(day);
        self
    }

    /// Use weekday value
    pub fn with_weekday(mut self, weekday: Weekday) -> Self {
        self.weekday = Some(weekday);
        self
    }

}

impl<I2C, E> PCF8563<I2C>
where
    I2C: I2c<Error = E>, 
{
    /// Set the alarm minutes [0-59], keeping the AE bit unchanged.
    pub async fn set_alarm_minutes(&mut self, minutes: u8) -> Result<(), Error<E>> {
        if minutes > 59 {
            return Err(Error::InvalidInputData);
        }
        self.set_alarm_value(Register::MINUTE_ALARM, minutes).await
    }

    /// Set the alarm hours [0-23], keeping the AE bit unchanged.
    pub async fn set_alarm_hours(&mut self, hours: u8) -> Result<(), Error<E>> {
        if hours > 23 {
            return Err(Error::InvalidInputData);
        }
        self.set_alarm_value(Register::HOUR_ALARM, hours).await
    }

    /// Set the alarm day [1-31], keeping the AE bit unchanged.
    pub async fn set_alarm_day(&mut self, day: u8) -> Result<(), Error<E>> {
        if day < 1 || day > 31 {
            return Err(Error::InvalidInputData);
        }
        self.set_alarm_value(Register::DAY_ALARM, day).await
    }

    /// Set the alarm weekday [0-6], keeping the AE bit unchanged.
    pub async fn set_alarm_weekday(&mut self, weekday: Weekday) -> Result<(), Error<E>> {
        self.set_alarm_value(Register::WEEKDAY_ALARM, weekday.value()).await
    }

    /// Control alarm minutes (On: alarm enabled, Off: alarm disabled).
    pub async fn control_alarm_minutes(&mut self, status: Control) -> Result<(), Error<E>> {
        self.control_bit_flag_inverted(Register::MINUTE_ALARM, BitFlags::AE, status).await
    }

    /// Is alarm minutes enabled?
    pub async fn is_alarm_minutes_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_alarm_enabled(Register::MINUTE_ALARM, BitFlags::AE).await
    }


    /// Control alarm hours (On: alarm enabled, Off: alarm disabled).
    pub async fn control_alarm_hours(&mut self, status: Control) -> Result<(), Error<E>> {
        self.control_bit_flag_inverted(Register::HOUR_ALARM,BitFlags::AE,status
        ).await
    }

    /// Is alarm hours enabled?
    pub async fn is_alarm_hours_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_alarm_enabled(Register::HOUR_ALARM, BitFlags::AE).await
    }

    /// Control alarm day (On: alarm enabled, Off: alarm disabled).
    pub async fn control_alarm_day(&mut self, status: Control) -> Result<(), Error<E>> {
        self.control_bit_flag_inverted(Register::DAY_ALARM, BitFlags::AE, status).await
    }

    /// Is alarm day enabled?
    pub async fn is_alarm_day_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_alarm_enabled(Register::DAY_ALARM, BitFlags::AE).await
    }

    /// Control alarm weekday (On: alarm enabled, Off: alarm disabled).
    pub async fn control_alarm_weekday(&mut self, status: Control) -> Result<(), Error<E>> {
        self.control_bit_flag_inverted(Register::WEEKDAY_ALARM, BitFlags::AE, status).await
    }

    /// Is alarm weekday enabled?
    pub async fn is_alarm_weekday_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_alarm_enabled(Register::WEEKDAY_ALARM, BitFlags::AE).await
    }

    /// Enable or disable alarm interrupt.
    pub async fn control_alarm_interrupt(&mut self, status: Control) -> Result<(), Error<E>> {
        self.control_bit_flag(Register::CTRL_STATUS_2, BitFlags::AIE, status).await
    }

    /// Read the alarm minutes setting.        
    pub async fn get_alarm_minutes(&mut self) -> Result<u8, Error<E>> {
        self.get_alarm_setting(Register::MINUTE_ALARM, 0b0111_1111).await
    }

    /// Read the alarm hours setting.
    pub async fn get_alarm_hours(&mut self) -> Result<u8, Error<E>> {
        self.get_alarm_setting(Register::HOUR_ALARM, 0b0011_1111).await
    }

    /// Read the alarm day setting.
    pub async fn get_alarm_day(&mut self) -> Result<u8, Error<E>> {
        self.get_alarm_setting(Register::DAY_ALARM, 0b0011_1111).await
    }

    /// Read the alarm weekday setting.
    pub async fn get_alarm_weekday(&mut self) -> Result<u8, Error<E>> {
        self.get_alarm_setting(Register::WEEKDAY_ALARM, 0b0000_0111).await
    }

    /// Get the alarm flag (if true, alarm event happened).
    pub async fn get_alarm_flag(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::CTRL_STATUS_2, BitFlags::AF).await
    }

    /// Clear the alarm flag.
    pub async fn clear_alarm_flag(&mut self) -> Result<(), Error<E>> {
        self.clear_register_bit_flag(Register::CTRL_STATUS_2, BitFlags::AF).await
    }

    /// Check if alarm interrupt is enabled.
    pub async fn is_alarm_interrupt_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::CTRL_STATUS_2, BitFlags::AIE).await
    }

    /// Shut off the alarms at once.
    pub async fn disable_all_alarms(&mut self) -> Result<(), Error<E>> {
        self.control_alarm_minutes(Control::Off).await?;
        self.control_alarm_hours(Control::Off).await?;
        self.control_alarm_day(Control::Off).await?;
        self.control_alarm_weekday(Control::Off).await?;
        Ok(())
    }

    /// Set the alarm in one go: minutes, hours, day, weekday
    /// All these values are optional, alarm only gets enabled for the provided data
    /// All previous settings are cleared    
    pub async fn set_alarm(&mut self, settings: &AlarmSettings) -> Result<(), Error<E>> {
        // check if input values make sense
        if let Some(minutes) = settings.minutes {
            if minutes > 59 {
                return Err(Error::InvalidInputData);
            }
        }

        if let Some(hours) = settings.hours {
            if hours > 23 {
                return Err(Error::InvalidInputData);
            }
        }

        if let Some(day) = settings.day {
            if day < 1 || day > 31 {
                return Err(Error::InvalidInputData);
            }
        }

        // clear the alarm flag
        self.clear_alarm_flag().await?;

        // disable all the alarms
        self.disable_all_alarms().await?;

        // set each components separately
        if let Some(minutes) = settings.minutes {
            self.set_alarm_minutes(minutes).await?;
            self.control_alarm_minutes(Control::On).await?;
        }
        if let Some(hours) = settings.hours {
            self.set_alarm_hours(hours).await?;
            self.control_alarm_hours(Control::On).await?;
        }
        if let Some(day) = settings.day {
            self.set_alarm_day(day).await?;
            self.control_alarm_day(Control::On).await?;
        }
        if let Some(weekday) = settings.weekday {
            self.set_alarm_weekday(weekday).await?;
            self.control_alarm_weekday(Control::On).await?;
        }
        Ok(())

    }

    /// Get the alarm settings: minutes, hours, day, weekday
    pub async fn get_alarm_settings(&mut self) -> Result<AlarmSettings, Error<E>> {
        let settings = AlarmSettings {
            minutes: if self.is_alarm_minutes_enabled().await? {
                Some(self.get_alarm_minutes().await?)
            } else {
                None
            },
            hours: if self.is_alarm_hours_enabled().await? {
                Some(self.get_alarm_hours().await?)
            } else {
                None
            },
            day: if self.is_alarm_day_enabled().await? {
                Some(self.get_alarm_day().await?)
            } else {
                None
            },
            weekday: if self.is_alarm_weekday_enabled().await? {
                Some(Weekday::try_from(self.get_alarm_weekday().await?).map_err(|_| Error::InvalidInputData)?)
            } else {
                None
            }
        };
        Ok(settings)
    }

    /// Is alarm enabled? Helper function.
    async fn is_alarm_enabled(
        &mut self,
        register: u8,
        bitmask: u8)
         -> Result<bool, Error<E>> {
        let flag = self.is_register_bit_flag_high(register, bitmask).await?;
        let flag = flag ^ true;
        Ok(flag)
    }
    
    /// Read the alarm setting. Helper function.
    async fn get_alarm_setting(
        &mut self,
        register: u8,
        decode_mask: u8
    ) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], &mut data)
            .await
            .map_err(Error::I2C)?;
        Ok(decode_bcd(data[0] & decode_mask))
    }

    /// Set alarm value. Helper function.
    async fn set_alarm_value(
        &mut self,
        register: u8,
        value: u8
    ) -> Result<(), Error<E>> {
        let data: u8 = self.read_register(register).await?; // read current value
        let data: u8 = (data & BitFlags::AE) | encode_bcd(value) ; // keep the AE bit as is        
        self.write_register(register, data).await
    }


}


