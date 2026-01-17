//! Various functions related to the RTC control that are not specifically
//! datetime-, timer-, alarm- or clock output-related will be defined here

use super::{I2c, BitFlags, Control, Error, Register, TimerFreq, PCF8563};


impl<I2C, E> PCF8563<I2C>
where
    I2C: I2c<Error = E>, 
{
    /// Enable or disable external clock test mode.
    pub async fn control_ext_clk_test_mode(&mut self, flag: Control) -> Result<(), Error<E>> {
        self.control_bit_flag(Register::CTRL_STATUS_1, BitFlags::TEST1, flag).await
        /*
        match flag {
            Control::On => self.set_register_bit_flag(Register::CTRL_STATUS_1, BitFlags::TEST1).await,
            Control::Off => self.clear_register_bit_flag(Register::CTRL_STATUS_1, BitFlags::TEST1).await,
        }
         */
        
    }

    /// Is the external clock test mode enabled?
    pub async fn is_ext_clk_mode_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::CTRL_STATUS_1, BitFlags::TEST1).await
    }

    /// Start/stop the internal clock.
    pub async fn control_clock(&mut self, flag: Control) -> Result<(), Error<E>> {
        self.control_bit_flag(Register::CTRL_STATUS_1, BitFlags::STOP, flag).await
        /*
        match flag {
            Control::On => self.clear_register_bit_flag(Register::CTRL_STATUS_1, BitFlags::STOP).await,
            Control::Off => self.set_register_bit_flag(Register::CTRL_STATUS_1, BitFlags::STOP).await,
        }
         */
    }

    /// Check if the internal clock is running.
    pub async fn is_clock_running(&mut self) -> Result<bool, Error<E>> {
        let flag = self.is_register_bit_flag_high(Register::CTRL_STATUS_1, BitFlags::STOP).await?;
        let flag = flag ^ true;
        Ok(flag)
    }

    /// Enable or disable power-on-reset override facility.
    pub async fn control_power_on_reset_override(&mut self, flag: Control) -> Result<(), Error<E>> {
        self.control_bit_flag(Register::CTRL_STATUS_1, BitFlags::TESTC, flag).await
        /*
        match flag {
            Control::On => self.set_register_bit_flag(Register::CTRL_STATUS_1, BitFlags::TESTC).await,
            Control::Off => self.clear_register_bit_flag(Register::CTRL_STATUS_1, BitFlags::TESTC).await,
        }
         */
    }

    /// Check if power-on-reset override facility is enabled.
    pub async fn is_power_on_reset_override_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::CTRL_STATUS_1, BitFlags::TESTC).await
    }

    /// Check the status of the Voltage Low detector flag
    pub async fn get_voltage_low_flag(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::VL_SECONDS, BitFlags::VL).await
    }

    /// Clear the voltage low detector flag.
    pub async fn clear_voltage_low_flag(&mut self) -> Result<(), Error<E>> {
        self.clear_register_bit_flag(Register::VL_SECONDS, BitFlags::VL).await
    }

    /// Initialize the RTC by setting all the control flags to zero, disabling alarms and timer, and setting the timer to the lowest frequency for power saving.
    pub async fn rtc_init(&mut self) -> Result<(), Error<E>> {
        self.write_register(Register::CTRL_STATUS_1, 0).await?; // clear all the control bits
        self.write_register(Register::CTRL_STATUS_2, 0).await?;
        self.clear_voltage_low_flag().await?; // clear the low voltage flag
        self.disable_all_alarms().await?; // disable alarm for all the components
        self.set_timer_frequency(TimerFreq::Timer_1_60Hz).await?; // set timer frequency to 1/60 Hz
        Ok(())
    }
}
