//! The game as shareware, with a BuildDeed license (NEAR testnet).
//!
//! An unregistered copy plays the first map: E1M1, or MAP01 of a Doom II
//! IWAD. A computer with a seat on a license plays every episode and map.
//! The license check comes from the BuildDeed SDK
//! (<https://github.com/13rac1/builddeed-sdk>): at start a worker thread
//! asks the license contract for the seat of this computer. R on the title
//! screen opens the register page and a worker thread waits for the seat.
//!
//! The check is in this GPL source code. Anyone may build the game without
//! it; the official build is the one that the license contract records.

use builddeed::{Client, Error, Product, REGISTER_WAIT, Seat};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// The license contract: the official BuildDeed collection on testnet.
pub const CONTRACT: &str = builddeed::OFFICIAL_TESTNET_CONTRACT;
/// The product key of the game in the contract and on the website.
pub const PRODUCT_KEY: &str = "doom";
/// The game version. The contract records the official files of each version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The message when an unregistered copy tries to start a map that is not free.
pub const REFUSAL: &str = "Register this computer to play this map.";
/// The message when an unregistered copy is given a patch WAD. A patch WAD
/// can put any map in the E1M1 slot, so only a registered copy loads one.
pub const PWAD_REFUSAL: &str = "Register this computer to load patch WADs.";

/// How often the worker thread asks for the seat while the register page is open.
const POLL_EVERY: Duration = Duration::from_secs(5);

/// The product that this game sells.
pub fn product() -> Product {
    Product::testnet(CONTRACT, PRODUCT_KEY, VERSION)
}

/// True for the map that an unregistered copy plays: map 1 of episode 1.
/// A Doom II IWAD has no episodes; the game gives episode 0 or 1 for it.
pub fn is_free_map(episode: usize, map: usize) -> bool {
    episode <= 1 && map == 1
}

/// The license state of this computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// The first check has not answered yet.
    Checking,
    /// The computer has a seat on a license that covers this version.
    Registered(Seat),
    /// No seat, or the check failed (see [`License::message`]).
    Unregistered,
    /// The register page is open; a worker thread waits for the seat.
    Waiting,
}

enum Msg {
    Check(Result<Option<Seat>, Error>),
    Registered(Result<Seat, Error>),
}

/// The license of this computer.
pub struct License {
    state: State,
    client: Client,
    message: String,
    stop: Option<Arc<AtomicBool>>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl License {
    /// Starts the license check of this computer for the game's product.
    pub fn start() -> Self {
        Self::start_with(Client::new(product()))
    }

    /// Starts the license check with `client`. Tests give a client with a
    /// fixed computer id and a fake RPC endpoint.
    pub fn start_with(client: Client) -> Self {
        let (tx, rx) = channel();
        let check = client.clone();
        let check_tx = tx.clone();
        std::thread::spawn(move || {
            // An error means that the game has stopped.
            check_tx.send(Msg::Check(check.check())).ok();
        });
        Self {
            state: State::Checking,
            client,
            message: String::new(),
            stop: None,
            tx,
            rx,
        }
    }

    /// The current state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// A line for the player, for example an error. Empty when there is none.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// True when the computer has a seat that covers this version.
    pub fn is_registered(&self) -> bool {
        matches!(self.state, State::Registered(_))
    }

    /// Applies the answers of the worker threads. Call it once per tic.
    pub fn poll(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.apply(msg);
        }
    }

    /// True when this copy may start the map. While the first check has not
    /// answered, this waits for it (at most the RPC time-out of the SDK).
    pub fn allows(&mut self, episode: usize, map: usize) -> bool {
        if is_free_map(episode, map) {
            return true;
        }
        self.wait_for_check();
        self.is_registered()
    }

    /// True when this copy may load patch WADs: only with a seat. This
    /// waits for the first check, as [`License::allows`] does.
    pub fn allows_pwads(&mut self) -> bool {
        self.wait_for_check();
        self.is_registered()
    }

    /// Shows [`REFUSAL`] on the license screen.
    pub fn refuse(&mut self) {
        REFUSAL.clone_into(&mut self.message);
    }

    /// Shows [`PWAD_REFUSAL`] on the license screen.
    pub fn refuse_pwads(&mut self) {
        PWAD_REFUSAL.clone_into(&mut self.message);
    }

    /// R on the title screen: opens the register page and waits for the
    /// seat. While the game waits, R stops the wait.
    pub fn register(&mut self) {
        if let Some(stop) = self.stop.take() {
            stop.store(true, Ordering::Relaxed);
            self.state = State::Unregistered;
            self.message.clear();
            return;
        }
        if self.is_registered() {
            return;
        }
        if let Err(e) = self.client.open_register_page() {
            self.message = e.to_string();
            return;
        }
        self.wait_for_seat(POLL_EVERY);
    }

    /// B on the title screen: opens the buy page of the product.
    pub fn buy(&mut self) {
        if let Err(e) = self.client.open_buy_page() {
            self.message = e.to_string();
        }
    }

    /// The lines of the license screen.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = match &self.state {
            State::Checking => vec!["Checking the license...".to_owned()],
            State::Registered(seat) => vec![
                format!("Registered to {}", seat.account_id),
                format!("Updates until {}", date(seat.updates_until_ms)),
            ],
            State::Unregistered => vec![
                "Unregistered shareware copy".to_owned(),
                format!("Computer {}", self.short_device()),
                "R: register this computer, B: buy a license".to_owned(),
            ],
            State::Waiting => vec![
                "Register this computer on the website".to_owned(),
                format!("Computer {}", self.short_device()),
                format!("If the page asks for a code: {}", self.confirm_code()),
                "Waiting for the seat...   R: stop".to_owned(),
            ],
        };
        if !self.message.is_empty() {
            lines.push(self.message.clone());
        }
        lines
    }

    /// The first and last characters of this computer's id, as the website
    /// shows them, or "unknown".
    fn short_device(&self) -> String {
        self.client
            .short_device_id()
            .unwrap_or_else(|_| "unknown".into())
    }

    /// The 6-digit code that the register page asks for when the browser
    /// blocks its check of this program, or "unknown".
    fn confirm_code(&self) -> String {
        self.client
            .confirm_code()
            .unwrap_or_else(|_| "unknown".into())
    }

    fn wait_for_check(&mut self) {
        while self.state == State::Checking {
            match self.rx.recv() {
                Ok(msg) => self.apply(msg),
                // Not possible: `self.tx` keeps the channel open.
                Err(_) => self.state = State::Unregistered,
            }
        }
    }

    /// Starts the worker thread that asks for the seat every `every`.
    fn wait_for_seat(&mut self, every: Duration) {
        let stop = Arc::new(AtomicBool::new(false));
        self.stop = Some(Arc::clone(&stop));
        self.state = State::Waiting;
        self.message.clear();
        let client = self.client.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let result = client.wait_for_seat(&stop, every, REGISTER_WAIT);
            if !stop.load(Ordering::Relaxed) {
                // An error means that the game has stopped.
                tx.send(Msg::Registered(result)).ok();
            }
        });
    }

    fn apply(&mut self, msg: Msg) {
        match msg {
            Msg::Check(Ok(Some(seat))) => {
                // A seat ends any wait for one.
                if let Some(stop) = self.stop.take() {
                    stop.store(true, Ordering::Relaxed);
                }
                self.state = State::Registered(seat);
            }
            // The start check can answer after the player pressed R: keep
            // waiting for the seat.
            Msg::Check(_) if self.state == State::Waiting => {}
            Msg::Check(Ok(None)) => self.state = State::Unregistered,
            Msg::Check(Err(e)) => {
                self.message = format!("Cannot check the license: {e}");
                self.state = State::Unregistered;
            }
            Msg::Registered(Ok(seat)) => {
                self.stop = None;
                self.message.clear();
                self.state = State::Registered(seat);
            }
            Msg::Registered(Err(e)) => {
                self.stop = None;
                self.message = e.to_string();
                self.state = State::Unregistered;
            }
        }
    }
}

/// UTC date as YYYY-MM-DD.
#[expect(
    clippy::cast_possible_wrap,
    reason = "a u64 count of days since 1970 fits an i64"
)]
fn date(ms: u64) -> String {
    let days = (ms / 86_400_000) as i64;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use builddeed::fake_rpc;
    use std::time::Instant;

    /// A license that asks the fake RPC server with these replies.
    fn license(replies: Vec<(u16, String)>) -> License {
        let product = product().with_rpc_url(fake_rpc::serve(replies));
        License::start_with(Client::with_device(product, "a".repeat(64)))
    }

    /// Polls until `done` or 5 seconds.
    fn poll_until(license: &mut License, done: impl Fn(&License) -> bool) {
        let end = Instant::now() + Duration::from_secs(5);
        while !done(license) && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(10));
            license.poll();
        }
    }

    #[test]
    fn product_is_doom_on_the_official_testnet_contract() {
        let p = product();
        assert_eq!(p.contract, "licenses.builddeed.testnet");
        assert_eq!(p.key, "doom");
        assert_eq!(p.version, VERSION);
        let c = Client::with_device(p, "a".repeat(64));
        assert!(
            c.register_url().unwrap().starts_with(
                "https://13rac1.github.io/builddeed/register.html#product=doom&device="
            ),
            "the register page of the product"
        );
    }

    #[test]
    fn only_the_first_map_is_free() {
        assert!(is_free_map(1, 1), "E1M1");
        assert!(is_free_map(0, 1), "MAP01 of Doom II");
        assert!(!is_free_map(1, 2), "E1M2");
        assert!(!is_free_map(2, 1), "E2M1");
        assert!(!is_free_map(0, 2), "MAP02");
    }

    #[test]
    fn unregistered_plays_only_the_first_map() {
        let mut l = license(vec![fake_rpc::no_seat()]);
        assert!(l.allows(1, 1), "E1M1 is free");
        assert!(!l.allows(1, 2), "E1M2 needs a seat");
        assert!(!l.allows(4, 1), "E4M1 needs a seat");
        assert_eq!(l.state(), &State::Unregistered);
        let lines = l.lines();
        assert_eq!(lines[0], "Unregistered shareware copy");
        assert_eq!(lines[1], "Computer aaaaaaaa...aaaa");
        assert_eq!(lines[2], "R: register this computer, B: buy a license");
        l.refuse();
        assert_eq!(l.lines().last().unwrap(), REFUSAL);
    }

    #[test]
    fn a_seat_opens_every_map() {
        let mut l = license(vec![fake_rpc::seat("doom:1", "alice.testnet", true)]);
        assert!(l.allows(3, 5), "E3M5 with a seat");
        assert!(l.is_registered());
        assert_eq!(
            l.lines(),
            vec!["Registered to alice.testnet", "Updates until 2027-11-10"]
        );
    }

    #[test]
    fn unregistered_loads_no_patch_wads() {
        let mut l = license(vec![fake_rpc::no_seat()]);
        assert!(!l.allows_pwads(), "no seat: no patch WADs");
        l.refuse_pwads();
        assert_eq!(l.lines().last().unwrap(), PWAD_REFUSAL);
    }

    #[test]
    fn a_seat_loads_patch_wads() {
        let mut l = license(vec![fake_rpc::seat("doom:1", "alice.testnet", true)]);
        assert!(l.allows_pwads(), "a seat: patch WADs load");
    }

    #[test]
    fn a_seat_that_does_not_cover_this_version_is_unregistered() {
        let mut l = license(vec![fake_rpc::seat("doom:1", "alice.testnet", false)]);
        assert!(!l.allows(1, 2), "the license ended before this version");
    }

    #[test]
    fn a_failed_check_runs_unregistered_and_says_why() {
        let mut l = license(vec![(500, "{}".to_owned())]);
        assert!(!l.allows(2, 2), "no network: unregistered");
        assert!(l.message().starts_with("Cannot check the license"));
    }

    #[test]
    fn the_wait_finds_the_seat() {
        let mut l = license(vec![
            fake_rpc::no_seat(),
            fake_rpc::no_seat(),
            fake_rpc::seat("doom:2", "bob.testnet", true),
        ]);
        poll_until(&mut l, |l| l.state() != &State::Checking);
        l.wait_for_seat(Duration::from_millis(10));
        assert_eq!(l.state(), &State::Waiting);
        assert!(l.lines()[2].starts_with("If the page asks for a code: "));
        poll_until(&mut l, License::is_registered);
        assert!(
            matches!(l.state(), State::Registered(s) if s.account_id == "bob.testnet"),
            "registered after the wait"
        );
    }

    #[test]
    fn r_stops_the_wait() {
        let mut l = license(vec![fake_rpc::no_seat()]);
        poll_until(&mut l, |l| l.state() != &State::Checking);
        l.wait_for_seat(Duration::from_secs(60));
        l.register();
        assert_eq!(l.state(), &State::Unregistered);
    }

    #[test]
    fn date_formats_utc_days() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(951_782_400_000), "2000-02-29");
    }
}
