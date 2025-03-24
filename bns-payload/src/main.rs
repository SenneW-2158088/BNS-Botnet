#[cfg(target_os = "unix")]
fn annoying_notification() {
    use notify_rust::{Hint, Notification};
    let bible_verses = [
        "For I know the plans I have for you, declares the Lord, plans for welfare and not for evil, to give you a future and a hope. - Jeremiah 29:11",
        "I can do all things through him who strengthens me. - Philippians 4:13",
        "But the fruit of the Spirit is love, joy, peace, forbearance, kindness, goodness, faithfulness, gentleness and self-control. - Galatians 5:22-23",
        "The Lord is my shepherd; I shall not want. - Psalm 23:1",
        "Trust in the Lord with all your heart, and do not lean on your own understanding. - Proverbs 3:5",
        "Be strong and courageous. Do not be afraid; do not be discouraged, for the Lord your God will be with you wherever you go. - Joshua 1:9",
        "The Lord is my light and my salvation; whom shall I fear? The Lord is the stronghold of my life; of whom shall I be afraid? - Psalm 27:1",
        "And we know that in all things God works for the good of those who love him, who have been called according to his purpose. - Romans 8:28",
        "But seek first his kingdom and his righteousness, and all these things will be given to you as well. - Matthew 6:33",
        "Come to me, all you who are weary and burdened, and I will give you rest. - Matthew 11:28",
        "The name of the Lord is a fortified tower; the righteous run to it and are safe. - Proverbs 18:10",
        "Cast all your anxiety on him because he cares for you. - 1 Peter 5:7",
        "The Lord is close to the brokenhearted and saves those who are crushed in spirit. - Psalm 34:18",
        "Do not be anxious about anything, but in every situation, by prayer and petition, with thanksgiving, present your requests to God. - Philippians 4:6",
        "For the Spirit God gave us does not make us timid, but gives us power, love and self-discipline. - 2 Timothy 1:7",
    ];
    let verse = bible_verses[(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % bible_verses.len() as u64) as usize];

    Notification::new()
        .summary("Thy Lord Hath Spoken! Hear His Words of Wisdom and Guidance")
        .body(verse)
        .icon("dialog-information")
        .hint(Hint::Resident(true)) // this is not supported by all implementations
        .timeout(0) // this however is
        .urgency(notify_rust::Urgency::Critical)
        .finalize()
        .show()
        .unwrap()
        .on_close(|| {
            annoying_notification();
        });
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) {
    use std::process::Command;
    let _ = Command::new("open").arg(url).status();
}

#[cfg(target_os = "macos")]
fn main() {
    open_browser(
        "https://www.uhasselt.be/en/instituten-en/expertise-centre-for-digital-media/research/networked-and-secure-systems",
    );
}

#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    annoying_notification();
}
