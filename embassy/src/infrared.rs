#[derive(Eq, PartialEq, Debug)]
enum BitPeriodState {
    HighStart,
    LowStart,
}

#[derive(Eq, PartialEq, Debug)]
enum State {
    Idle,
    MaybePreamble {
        start_high_at: u64,
    },
    ReadyAfterPreamble,
    BitPeriod {
        high_start_at: u64,
        low_start_at: u64,
        low_end_at: u64,
        state: BitPeriodState,
    },
}

pub struct IrDecoder {
    state: State,
}

impl IrDecoder {
    pub fn new() -> Self {
        Self {
            state: State::Idle,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    DecodedBit(bool),
    DecodedUnknown,
    NewFrameReady,
    DataFragReady,
}

impl IrDecoder {
    /// `now` is in micros
    ///
    /// Emits bit result, if any.
    pub fn tick(&mut self, signal: bool, now: u64) -> Option<Event> {
        let mut result = None;

        const PREAMBLE_DURATION_RANGE: (u64, u64) = (8000, 10000) /* 8ms - 10ms */;
        match self.state {
            State::Idle => {
                if signal {
                    self.state = State::MaybePreamble { start_high_at: now };
                }
            }
            State::MaybePreamble { start_high_at } => {
                if !signal {
                    if now - start_high_at >= PREAMBLE_DURATION_RANGE.0
                        && now - start_high_at <= PREAMBLE_DURATION_RANGE.1
                    {
                        // valid preamble
                        self.state = State::ReadyAfterPreamble;
                        result = Some(Event::NewFrameReady);
                    } else {
                        // ignore this noise and reset to idle
                        self.state = State::Idle;
                    }
                }
            }
            State::ReadyAfterPreamble => {
                if signal {
                    // start recording this bit period
                    self.state = State::BitPeriod {
                        high_start_at: now,
                        low_start_at: 0,
                        low_end_at: 0,
                        state: BitPeriodState::HighStart,
                    };
                }
            }
            State::BitPeriod {
                ref mut high_start_at,
                ref mut low_start_at,
                ref mut low_end_at,
                state: ref mut period_state,
            } => {
                match period_state {
                    BitPeriodState::HighStart => {
                        if !signal {
                            *period_state = BitPeriodState::LowStart;
                            *low_start_at = now;
                        } else {
                            // if the high state goes so long (> 8ms) then it's probably a preamble;
                            // then force terminate it and go to MaybePreamble
                            if now - *high_start_at > 8_000
                            /* 8ms */
                            {
                                self.state = State::MaybePreamble {
                                    start_high_at: *high_start_at,
                                };
                            }
                        }
                    }
                    BitPeriodState::LowStart => {
                        // a new period starts
                        // decode the old period and start a new period
                        if signal {
                            *period_state = BitPeriodState::HighStart;
                            *low_end_at = now;

                            let low_duration = *low_end_at - *low_start_at;
                            let high_duration = *low_start_at - *high_start_at;
                            // let ratio = low_duration as f64 / high_duration as f64;
                            let ratio = low_duration as f64 / 562.5;
                            if (0.5..=2.0).contains(&ratio) {
                                // logical bit 0
                                result = Some(Event::DecodedBit(false));
                            } else if (2.1..=3.5).contains(&ratio) {
                                // logical bit 1
                                result = Some(Event::DecodedBit(true));
                            } else {
                                // invalid encoding
                                result = Some(Event::DecodedUnknown);
                            }

                            *high_start_at = now;
                        } else {
                            // if the low state keeps too long then it indicates the end; reset to ready
                            if now - *low_start_at > 6_000
                            /* 6ms */
                            {
                                self.state = State::ReadyAfterPreamble;
                                result = Some(Event::DataFragReady);
                            }
                        }
                    }
                }
            }
        }
        result
    }
}

