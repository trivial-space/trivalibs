/// Custom event types for painter, independent of winit

/// Pointer button types - device-agnostic naming
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerButton {
	Primary,
	Secondary,
	Middle,
	Other(u16),
}

impl From<winit::event::MouseButton> for PointerButton {
	fn from(button: winit::event::MouseButton) -> Self {
		match button {
			winit::event::MouseButton::Left => PointerButton::Primary,
			winit::event::MouseButton::Right => PointerButton::Secondary,
			winit::event::MouseButton::Middle => PointerButton::Middle,
			winit::event::MouseButton::Back => PointerButton::Other(4),
			winit::event::MouseButton::Forward => PointerButton::Other(5),
			winit::event::MouseButton::Other(n) => PointerButton::Other(n),
		}
	}
}

/// Physical key codes - crate-local copy of winit's KeyCode
/// This allows WASM builds to emit events without depending on winit
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
	/// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
	/// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
	/// (hankaku/zenkaku/kanji) key on Japanese keyboards.
	Backquote,
	/// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
	/// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
	/// 104- and 106-key layouts.
	/// Labeled <kbd>#</kbd> on a UK (102) keyboard.
	Backslash,
	/// <kbd>[</kbd> on a US keyboard.
	BracketLeft,
	/// <kbd>]</kbd> on a US keyboard.
	BracketRight,
	/// <kbd>,</kbd> on a US keyboard.
	Comma,
	/// <kbd>0</kbd> on a US keyboard.
	Digit0,
	/// <kbd>1</kbd> on a US keyboard.
	Digit1,
	/// <kbd>2</kbd> on a US keyboard.
	Digit2,
	/// <kbd>3</kbd> on a US keyboard.
	Digit3,
	/// <kbd>4</kbd> on a US keyboard.
	Digit4,
	/// <kbd>5</kbd> on a US keyboard.
	Digit5,
	/// <kbd>6</kbd> on a US keyboard.
	Digit6,
	/// <kbd>7</kbd> on a US keyboard.
	Digit7,
	/// <kbd>8</kbd> on a US keyboard.
	Digit8,
	/// <kbd>9</kbd> on a US keyboard.
	Digit9,
	/// <kbd>=</kbd> on a US keyboard.
	Equal,
	/// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
	/// Labeled <kbd>\\</kbd> on a UK keyboard.
	IntlBackslash,
	/// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
	/// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
	IntlRo,
	/// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
	/// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
	/// Russian keyboard.
	IntlYen,
	/// <kbd>a</kbd> on a US keyboard.
	/// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
	KeyA,
	/// <kbd>b</kbd> on a US keyboard.
	KeyB,
	/// <kbd>c</kbd> on a US keyboard.
	KeyC,
	/// <kbd>d</kbd> on a US keyboard.
	KeyD,
	/// <kbd>e</kbd> on a US keyboard.
	KeyE,
	/// <kbd>f</kbd> on a US keyboard.
	KeyF,
	/// <kbd>g</kbd> on a US keyboard.
	KeyG,
	/// <kbd>h</kbd> on a US keyboard.
	KeyH,
	/// <kbd>i</kbd> on a US keyboard.
	KeyI,
	/// <kbd>j</kbd> on a US keyboard.
	KeyJ,
	/// <kbd>k</kbd> on a US keyboard.
	KeyK,
	/// <kbd>l</kbd> on a US keyboard.
	KeyL,
	/// <kbd>m</kbd> on a US keyboard.
	KeyM,
	/// <kbd>n</kbd> on a US keyboard.
	KeyN,
	/// <kbd>o</kbd> on a US keyboard.
	KeyO,
	/// <kbd>p</kbd> on a US keyboard.
	KeyP,
	/// <kbd>q</kbd> on a US keyboard.
	/// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
	KeyQ,
	/// <kbd>r</kbd> on a US keyboard.
	KeyR,
	/// <kbd>s</kbd> on a US keyboard.
	KeyS,
	/// <kbd>t</kbd> on a US keyboard.
	KeyT,
	/// <kbd>u</kbd> on a US keyboard.
	KeyU,
	/// <kbd>v</kbd> on a US keyboard.
	KeyV,
	/// <kbd>w</kbd> on a US keyboard.
	/// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
	KeyW,
	/// <kbd>x</kbd> on a US keyboard.
	KeyX,
	/// <kbd>y</kbd> on a US keyboard.
	/// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
	KeyY,
	/// <kbd>z</kbd> on a US keyboard.
	/// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
	/// QWERTZ (e.g., German) keyboard.
	KeyZ,
	/// <kbd>-</kbd> on a US keyboard.
	Minus,
	/// <kbd>.</kbd> on a US keyboard.
	Period,
	/// <kbd>'</kbd> on a US keyboard.
	Quote,
	/// <kbd>;</kbd> on a US keyboard.
	Semicolon,
	/// <kbd>/</kbd> on a US keyboard.
	Slash,
	/// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
	AltLeft,
	/// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
	/// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
	AltRight,
	/// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
	/// Labeled <kbd>Delete</kbd> on Apple keyboards.
	Backspace,
	/// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
	CapsLock,
	/// The application context menu key, which is typically found between the right
	/// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
	ContextMenu,
	/// <kbd>Control</kbd> or <kbd>⌃</kbd>
	ControlLeft,
	/// <kbd>Control</kbd> or <kbd>⌃</kbd>
	ControlRight,
	/// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
	Enter,
	/// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
	SuperLeft,
	/// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
	SuperRight,
	/// <kbd>Shift</kbd> or <kbd>⇧</kbd>
	ShiftLeft,
	/// <kbd>Shift</kbd> or <kbd>⇧</kbd>
	ShiftRight,
	/// <kbd> </kbd> (space)
	Space,
	/// <kbd>Tab</kbd> or <kbd>⇥</kbd>
	Tab,
	/// Japanese: <kbd>変</kbd> (henkan)
	Convert,
	/// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
	KanaMode,
	/// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
	///
	/// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
	Lang1,
	/// Korean: Hanja <kbd>한</kbd> (hanja)
	///
	/// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
	Lang2,
	/// Japanese (word-processing keyboard): Katakana
	Lang3,
	/// Japanese (word-processing keyboard): Hiragana
	Lang4,
	/// Japanese (word-processing keyboard): Zenkaku/Hankaku
	Lang5,
	/// Japanese: <kbd>無変換</kbd> (muhenkan)
	NonConvert,
	/// <kbd>⌦</kbd>. The forward delete key.
	/// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
	/// the keyboard is encoded as [`Backspace`].
	///
	/// [`Backspace`]: Self::Backspace
	Delete,
	/// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
	End,
	/// <kbd>Help</kbd>. Not present on standard PC keyboards.
	Help,
	/// <kbd>Home</kbd> or <kbd>↖</kbd>
	Home,
	/// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
	Insert,
	/// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
	PageDown,
	/// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
	PageUp,
	/// <kbd>↓</kbd>
	ArrowDown,
	/// <kbd>←</kbd>
	ArrowLeft,
	/// <kbd>→</kbd>
	ArrowRight,
	/// <kbd>↑</kbd>
	ArrowUp,
	/// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
	NumLock,
	/// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
	Numpad0,
	/// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
	Numpad1,
	/// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
	Numpad2,
	/// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
	Numpad3,
	/// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
	Numpad4,
	/// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
	Numpad5,
	/// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
	Numpad6,
	/// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
	/// or remote control
	Numpad7,
	/// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
	Numpad8,
	/// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
	/// or remote control
	Numpad9,
	/// <kbd>+</kbd>
	NumpadAdd,
	/// Found on the Microsoft Natural Keyboard.
	NumpadBackspace,
	/// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
	/// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
	/// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
	///
	/// [`NumLock`]: Self::NumLock
	NumpadClear,
	/// <kbd>C</kbd> (Clear Entry)
	NumpadClearEntry,
	/// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
	/// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
	NumpadComma,
	/// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
	/// Brazil), this key may generate a <kbd>,</kbd>.
	NumpadDecimal,
	/// <kbd>/</kbd>
	NumpadDivide,
	NumpadEnter,
	/// <kbd>=</kbd>
	NumpadEqual,
	/// <kbd>#</kbd> on a phone or remote control device. This key is typically found
	/// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
	NumpadHash,
	/// <kbd>M</kbd> Add current entry to the value stored in memory.
	NumpadMemoryAdd,
	/// <kbd>M</kbd> Clear the value stored in memory.
	NumpadMemoryClear,
	/// <kbd>M</kbd> Replace the current entry with the value stored in memory.
	NumpadMemoryRecall,
	/// <kbd>M</kbd> Replace the value stored in memory with the current entry.
	NumpadMemoryStore,
	/// <kbd>M</kbd> Subtract current entry from the value stored in memory.
	NumpadMemorySubtract,
	/// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
	/// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
	///
	/// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
	NumpadMultiply,
	/// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
	NumpadParenLeft,
	/// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
	NumpadParenRight,
	/// <kbd>*</kbd> on a phone or remote control device.
	///
	/// This key is typically found below the <kbd>7</kbd> key and to the left of
	/// the <kbd>0</kbd> key.
	///
	/// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
	/// numeric keypads.
	NumpadStar,
	/// <kbd>-</kbd>
	NumpadSubtract,
	/// <kbd>Esc</kbd> or <kbd>⎋</kbd>
	Escape,
	/// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
	Fn,
	/// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
	/// Natural Keyboard.
	FnLock,
	/// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
	PrintScreen,
	/// <kbd>Scroll Lock</kbd>
	ScrollLock,
	/// <kbd>Pause Break</kbd>
	Pause,
	/// Some laptops place this key to the left of the <kbd>↑</kbd> key.
	///
	/// This also the "back" button (triangle) on Android.
	BrowserBack,
	BrowserFavorites,
	/// Some laptops place this key to the right of the <kbd>↑</kbd> key.
	BrowserForward,
	/// The "home" button on Android.
	BrowserHome,
	BrowserRefresh,
	BrowserSearch,
	BrowserStop,
	/// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
	/// keyboards.
	Eject,
	/// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
	LaunchApp1,
	/// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
	LaunchApp2,
	LaunchMail,
	MediaPlayPause,
	MediaSelect,
	MediaStop,
	MediaTrackNext,
	MediaTrackPrevious,
	/// This key is placed in the function section on some Apple keyboards, replacing the
	/// <kbd>Eject</kbd> key.
	Power,
	Sleep,
	AudioVolumeDown,
	AudioVolumeMute,
	AudioVolumeUp,
	WakeUp,
	// Legacy modifier key. Also called "Super" in certain places.
	Meta,
	// Legacy modifier key.
	Hyper,
	Turbo,
	Abort,
	Resume,
	Suspend,
	/// Found on Sun's USB keyboard.
	Again,
	/// Found on Sun's USB keyboard.
	Copy,
	/// Found on Sun's USB keyboard.
	Cut,
	/// Found on Sun's USB keyboard.
	Find,
	/// Found on Sun's USB keyboard.
	Open,
	/// Found on Sun's USB keyboard.
	Paste,
	/// Found on Sun's USB keyboard.
	Props,
	/// Found on Sun's USB keyboard.
	Select,
	/// Found on Sun's USB keyboard.
	Undo,
	/// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
	Hiragana,
	/// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
	Katakana,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F1,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F2,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F3,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F4,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F5,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F6,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F7,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F8,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F9,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F10,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F11,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F12,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F13,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F14,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F15,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F16,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F17,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F18,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F19,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F20,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F21,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F22,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F23,
	/// General-purpose function key.
	/// Usually found at the top of the keyboard.
	F24,
	/// General-purpose function key.
	F25,
	/// General-purpose function key.
	F26,
	/// General-purpose function key.
	F27,
	/// General-purpose function key.
	F28,
	/// General-purpose function key.
	F29,
	/// General-purpose function key.
	F30,
	/// General-purpose function key.
	F31,
	/// General-purpose function key.
	F32,
	/// General-purpose function key.
	F33,
	/// General-purpose function key.
	F34,
	/// General-purpose function key.
	F35,
}

impl From<winit::keyboard::KeyCode> for KeyCode {
	fn from(key: winit::keyboard::KeyCode) -> Self {
		match key {
			winit::keyboard::KeyCode::Backquote => KeyCode::Backquote,
			winit::keyboard::KeyCode::Backslash => KeyCode::Backslash,
			winit::keyboard::KeyCode::BracketLeft => KeyCode::BracketLeft,
			winit::keyboard::KeyCode::BracketRight => KeyCode::BracketRight,
			winit::keyboard::KeyCode::Comma => KeyCode::Comma,
			winit::keyboard::KeyCode::Digit0 => KeyCode::Digit0,
			winit::keyboard::KeyCode::Digit1 => KeyCode::Digit1,
			winit::keyboard::KeyCode::Digit2 => KeyCode::Digit2,
			winit::keyboard::KeyCode::Digit3 => KeyCode::Digit3,
			winit::keyboard::KeyCode::Digit4 => KeyCode::Digit4,
			winit::keyboard::KeyCode::Digit5 => KeyCode::Digit5,
			winit::keyboard::KeyCode::Digit6 => KeyCode::Digit6,
			winit::keyboard::KeyCode::Digit7 => KeyCode::Digit7,
			winit::keyboard::KeyCode::Digit8 => KeyCode::Digit8,
			winit::keyboard::KeyCode::Digit9 => KeyCode::Digit9,
			winit::keyboard::KeyCode::Equal => KeyCode::Equal,
			winit::keyboard::KeyCode::IntlBackslash => KeyCode::IntlBackslash,
			winit::keyboard::KeyCode::IntlRo => KeyCode::IntlRo,
			winit::keyboard::KeyCode::IntlYen => KeyCode::IntlYen,
			winit::keyboard::KeyCode::KeyA => KeyCode::KeyA,
			winit::keyboard::KeyCode::KeyB => KeyCode::KeyB,
			winit::keyboard::KeyCode::KeyC => KeyCode::KeyC,
			winit::keyboard::KeyCode::KeyD => KeyCode::KeyD,
			winit::keyboard::KeyCode::KeyE => KeyCode::KeyE,
			winit::keyboard::KeyCode::KeyF => KeyCode::KeyF,
			winit::keyboard::KeyCode::KeyG => KeyCode::KeyG,
			winit::keyboard::KeyCode::KeyH => KeyCode::KeyH,
			winit::keyboard::KeyCode::KeyI => KeyCode::KeyI,
			winit::keyboard::KeyCode::KeyJ => KeyCode::KeyJ,
			winit::keyboard::KeyCode::KeyK => KeyCode::KeyK,
			winit::keyboard::KeyCode::KeyL => KeyCode::KeyL,
			winit::keyboard::KeyCode::KeyM => KeyCode::KeyM,
			winit::keyboard::KeyCode::KeyN => KeyCode::KeyN,
			winit::keyboard::KeyCode::KeyO => KeyCode::KeyO,
			winit::keyboard::KeyCode::KeyP => KeyCode::KeyP,
			winit::keyboard::KeyCode::KeyQ => KeyCode::KeyQ,
			winit::keyboard::KeyCode::KeyR => KeyCode::KeyR,
			winit::keyboard::KeyCode::KeyS => KeyCode::KeyS,
			winit::keyboard::KeyCode::KeyT => KeyCode::KeyT,
			winit::keyboard::KeyCode::KeyU => KeyCode::KeyU,
			winit::keyboard::KeyCode::KeyV => KeyCode::KeyV,
			winit::keyboard::KeyCode::KeyW => KeyCode::KeyW,
			winit::keyboard::KeyCode::KeyX => KeyCode::KeyX,
			winit::keyboard::KeyCode::KeyY => KeyCode::KeyY,
			winit::keyboard::KeyCode::KeyZ => KeyCode::KeyZ,
			winit::keyboard::KeyCode::Minus => KeyCode::Minus,
			winit::keyboard::KeyCode::Period => KeyCode::Period,
			winit::keyboard::KeyCode::Quote => KeyCode::Quote,
			winit::keyboard::KeyCode::Semicolon => KeyCode::Semicolon,
			winit::keyboard::KeyCode::Slash => KeyCode::Slash,
			winit::keyboard::KeyCode::AltLeft => KeyCode::AltLeft,
			winit::keyboard::KeyCode::AltRight => KeyCode::AltRight,
			winit::keyboard::KeyCode::Backspace => KeyCode::Backspace,
			winit::keyboard::KeyCode::CapsLock => KeyCode::CapsLock,
			winit::keyboard::KeyCode::ContextMenu => KeyCode::ContextMenu,
			winit::keyboard::KeyCode::ControlLeft => KeyCode::ControlLeft,
			winit::keyboard::KeyCode::ControlRight => KeyCode::ControlRight,
			winit::keyboard::KeyCode::Enter => KeyCode::Enter,
			winit::keyboard::KeyCode::SuperLeft => KeyCode::SuperLeft,
			winit::keyboard::KeyCode::SuperRight => KeyCode::SuperRight,
			winit::keyboard::KeyCode::ShiftLeft => KeyCode::ShiftLeft,
			winit::keyboard::KeyCode::ShiftRight => KeyCode::ShiftRight,
			winit::keyboard::KeyCode::Space => KeyCode::Space,
			winit::keyboard::KeyCode::Tab => KeyCode::Tab,
			winit::keyboard::KeyCode::Convert => KeyCode::Convert,
			winit::keyboard::KeyCode::KanaMode => KeyCode::KanaMode,
			winit::keyboard::KeyCode::Lang1 => KeyCode::Lang1,
			winit::keyboard::KeyCode::Lang2 => KeyCode::Lang2,
			winit::keyboard::KeyCode::Lang3 => KeyCode::Lang3,
			winit::keyboard::KeyCode::Lang4 => KeyCode::Lang4,
			winit::keyboard::KeyCode::Lang5 => KeyCode::Lang5,
			winit::keyboard::KeyCode::NonConvert => KeyCode::NonConvert,
			winit::keyboard::KeyCode::Delete => KeyCode::Delete,
			winit::keyboard::KeyCode::End => KeyCode::End,
			winit::keyboard::KeyCode::Help => KeyCode::Help,
			winit::keyboard::KeyCode::Home => KeyCode::Home,
			winit::keyboard::KeyCode::Insert => KeyCode::Insert,
			winit::keyboard::KeyCode::PageDown => KeyCode::PageDown,
			winit::keyboard::KeyCode::PageUp => KeyCode::PageUp,
			winit::keyboard::KeyCode::ArrowDown => KeyCode::ArrowDown,
			winit::keyboard::KeyCode::ArrowLeft => KeyCode::ArrowLeft,
			winit::keyboard::KeyCode::ArrowRight => KeyCode::ArrowRight,
			winit::keyboard::KeyCode::ArrowUp => KeyCode::ArrowUp,
			winit::keyboard::KeyCode::NumLock => KeyCode::NumLock,
			winit::keyboard::KeyCode::Numpad0 => KeyCode::Numpad0,
			winit::keyboard::KeyCode::Numpad1 => KeyCode::Numpad1,
			winit::keyboard::KeyCode::Numpad2 => KeyCode::Numpad2,
			winit::keyboard::KeyCode::Numpad3 => KeyCode::Numpad3,
			winit::keyboard::KeyCode::Numpad4 => KeyCode::Numpad4,
			winit::keyboard::KeyCode::Numpad5 => KeyCode::Numpad5,
			winit::keyboard::KeyCode::Numpad6 => KeyCode::Numpad6,
			winit::keyboard::KeyCode::Numpad7 => KeyCode::Numpad7,
			winit::keyboard::KeyCode::Numpad8 => KeyCode::Numpad8,
			winit::keyboard::KeyCode::Numpad9 => KeyCode::Numpad9,
			winit::keyboard::KeyCode::NumpadAdd => KeyCode::NumpadAdd,
			winit::keyboard::KeyCode::NumpadBackspace => KeyCode::NumpadBackspace,
			winit::keyboard::KeyCode::NumpadClear => KeyCode::NumpadClear,
			winit::keyboard::KeyCode::NumpadClearEntry => KeyCode::NumpadClearEntry,
			winit::keyboard::KeyCode::NumpadComma => KeyCode::NumpadComma,
			winit::keyboard::KeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
			winit::keyboard::KeyCode::NumpadDivide => KeyCode::NumpadDivide,
			winit::keyboard::KeyCode::NumpadEnter => KeyCode::NumpadEnter,
			winit::keyboard::KeyCode::NumpadEqual => KeyCode::NumpadEqual,
			winit::keyboard::KeyCode::NumpadHash => KeyCode::NumpadHash,
			winit::keyboard::KeyCode::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
			winit::keyboard::KeyCode::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
			winit::keyboard::KeyCode::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
			winit::keyboard::KeyCode::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
			winit::keyboard::KeyCode::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
			winit::keyboard::KeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
			winit::keyboard::KeyCode::NumpadParenLeft => KeyCode::NumpadParenLeft,
			winit::keyboard::KeyCode::NumpadParenRight => KeyCode::NumpadParenRight,
			winit::keyboard::KeyCode::NumpadStar => KeyCode::NumpadStar,
			winit::keyboard::KeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
			winit::keyboard::KeyCode::Escape => KeyCode::Escape,
			winit::keyboard::KeyCode::Fn => KeyCode::Fn,
			winit::keyboard::KeyCode::FnLock => KeyCode::FnLock,
			winit::keyboard::KeyCode::PrintScreen => KeyCode::PrintScreen,
			winit::keyboard::KeyCode::ScrollLock => KeyCode::ScrollLock,
			winit::keyboard::KeyCode::Pause => KeyCode::Pause,
			winit::keyboard::KeyCode::BrowserBack => KeyCode::BrowserBack,
			winit::keyboard::KeyCode::BrowserFavorites => KeyCode::BrowserFavorites,
			winit::keyboard::KeyCode::BrowserForward => KeyCode::BrowserForward,
			winit::keyboard::KeyCode::BrowserHome => KeyCode::BrowserHome,
			winit::keyboard::KeyCode::BrowserRefresh => KeyCode::BrowserRefresh,
			winit::keyboard::KeyCode::BrowserSearch => KeyCode::BrowserSearch,
			winit::keyboard::KeyCode::BrowserStop => KeyCode::BrowserStop,
			winit::keyboard::KeyCode::Eject => KeyCode::Eject,
			winit::keyboard::KeyCode::LaunchApp1 => KeyCode::LaunchApp1,
			winit::keyboard::KeyCode::LaunchApp2 => KeyCode::LaunchApp2,
			winit::keyboard::KeyCode::LaunchMail => KeyCode::LaunchMail,
			winit::keyboard::KeyCode::MediaPlayPause => KeyCode::MediaPlayPause,
			winit::keyboard::KeyCode::MediaSelect => KeyCode::MediaSelect,
			winit::keyboard::KeyCode::MediaStop => KeyCode::MediaStop,
			winit::keyboard::KeyCode::MediaTrackNext => KeyCode::MediaTrackNext,
			winit::keyboard::KeyCode::MediaTrackPrevious => KeyCode::MediaTrackPrevious,
			winit::keyboard::KeyCode::Power => KeyCode::Power,
			winit::keyboard::KeyCode::Sleep => KeyCode::Sleep,
			winit::keyboard::KeyCode::AudioVolumeDown => KeyCode::AudioVolumeDown,
			winit::keyboard::KeyCode::AudioVolumeMute => KeyCode::AudioVolumeMute,
			winit::keyboard::KeyCode::AudioVolumeUp => KeyCode::AudioVolumeUp,
			winit::keyboard::KeyCode::WakeUp => KeyCode::WakeUp,
			winit::keyboard::KeyCode::Meta => KeyCode::Meta,
			winit::keyboard::KeyCode::Hyper => KeyCode::Hyper,
			winit::keyboard::KeyCode::Turbo => KeyCode::Turbo,
			winit::keyboard::KeyCode::Abort => KeyCode::Abort,
			winit::keyboard::KeyCode::Resume => KeyCode::Resume,
			winit::keyboard::KeyCode::Suspend => KeyCode::Suspend,
			winit::keyboard::KeyCode::Again => KeyCode::Again,
			winit::keyboard::KeyCode::Copy => KeyCode::Copy,
			winit::keyboard::KeyCode::Cut => KeyCode::Cut,
			winit::keyboard::KeyCode::Find => KeyCode::Find,
			winit::keyboard::KeyCode::Open => KeyCode::Open,
			winit::keyboard::KeyCode::Paste => KeyCode::Paste,
			winit::keyboard::KeyCode::Props => KeyCode::Props,
			winit::keyboard::KeyCode::Select => KeyCode::Select,
			winit::keyboard::KeyCode::Undo => KeyCode::Undo,
			winit::keyboard::KeyCode::Hiragana => KeyCode::Hiragana,
			winit::keyboard::KeyCode::Katakana => KeyCode::Katakana,
			winit::keyboard::KeyCode::F1 => KeyCode::F1,
			winit::keyboard::KeyCode::F2 => KeyCode::F2,
			winit::keyboard::KeyCode::F3 => KeyCode::F3,
			winit::keyboard::KeyCode::F4 => KeyCode::F4,
			winit::keyboard::KeyCode::F5 => KeyCode::F5,
			winit::keyboard::KeyCode::F6 => KeyCode::F6,
			winit::keyboard::KeyCode::F7 => KeyCode::F7,
			winit::keyboard::KeyCode::F8 => KeyCode::F8,
			winit::keyboard::KeyCode::F9 => KeyCode::F9,
			winit::keyboard::KeyCode::F10 => KeyCode::F10,
			winit::keyboard::KeyCode::F11 => KeyCode::F11,
			winit::keyboard::KeyCode::F12 => KeyCode::F12,
			winit::keyboard::KeyCode::F13 => KeyCode::F13,
			winit::keyboard::KeyCode::F14 => KeyCode::F14,
			winit::keyboard::KeyCode::F15 => KeyCode::F15,
			winit::keyboard::KeyCode::F16 => KeyCode::F16,
			winit::keyboard::KeyCode::F17 => KeyCode::F17,
			winit::keyboard::KeyCode::F18 => KeyCode::F18,
			winit::keyboard::KeyCode::F19 => KeyCode::F19,
			winit::keyboard::KeyCode::F20 => KeyCode::F20,
			winit::keyboard::KeyCode::F21 => KeyCode::F21,
			winit::keyboard::KeyCode::F22 => KeyCode::F22,
			winit::keyboard::KeyCode::F23 => KeyCode::F23,
			winit::keyboard::KeyCode::F24 => KeyCode::F24,
			winit::keyboard::KeyCode::F25 => KeyCode::F25,
			winit::keyboard::KeyCode::F26 => KeyCode::F26,
			winit::keyboard::KeyCode::F27 => KeyCode::F27,
			winit::keyboard::KeyCode::F28 => KeyCode::F28,
			winit::keyboard::KeyCode::F29 => KeyCode::F29,
			winit::keyboard::KeyCode::F30 => KeyCode::F30,
			winit::keyboard::KeyCode::F31 => KeyCode::F31,
			winit::keyboard::KeyCode::F32 => KeyCode::F32,
			winit::keyboard::KeyCode::F33 => KeyCode::F33,
			winit::keyboard::KeyCode::F34 => KeyCode::F34,
			winit::keyboard::KeyCode::F35 => KeyCode::F35,
			_ => KeyCode::Escape, // Fallback for any unmapped keys
		}
	}
}

/// Custom events for painter applications
#[derive(Debug)]
pub enum Event<UserEvent> {
	/// Pointer button pressed
	PointerDown {
		button: PointerButton,
		x: f64,
		y: f64,
	},
	/// Pointer button released
	PointerUp {
		button: PointerButton,
		x: f64,
		y: f64,
	},
	/// Pointer moved
	/// When mouse_lock is false: x/y are absolute cursor coordinates, delta_x/delta_y show movement
	/// When mouse_lock is true: x/y are 0, delta_x/delta_y contain raw motion deltas
	PointerMove {
		x: f64,
		y: f64,
		delta_x: f64,
		delta_y: f64,
		mouse_lock: bool,
	},
	/// Key pressed
	KeyDown { key: KeyCode },
	/// Key released
	KeyUp { key: KeyCode },
	/// User-defined event
	UserEvent(UserEvent),
	/// Shader file was reloaded (debug mode only)
	ShaderReloadEvent,
}
