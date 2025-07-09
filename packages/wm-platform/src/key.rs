use wm_macros::KeyConversions;

pub trait IsKeyDownRaw {
  /// Returns `true` if the key is currently pressed down.
  fn is_down_raw(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, KeyConversions)]
#[key(win = crate::raw::windows::key::WinKey, macos = NotYetImplemented, linux = crate::platform_impl::key::LinuxKey)]
pub enum Key {
  #[key("a", win = A, macos = !, linux = A)]
  A,
  #[key("abntc1", win = AbntC1, macos = !, linux = !)]
  AbntC1,
  #[key("abntc2", win = AbntC2, macos = !, linux = !)]
  AbntC2,
  #[key("accept", win = Accept, macos = !, linux = !)]
  Accept,
  #[key("add", win = Add, macos = !, linux = !)]
  Add,
  #[key("apps", win = Apps, macos = !, linux = !)]
  Apps,
  #[key("attn", win = Attn, macos = !, linux = !)]
  Attn,
  #[key("b", win = B, macos = !, linux = !)]
  B,
  #[key("back", win = Back, macos = !, linux = !)]
  Back,
  #[key("browser back", win = BrowserBack, macos = !, linux = !)]
  BrowserBack,
  #[key("browser favorites", win = BrowserFavorites, macos = !, linux = !)]
  BrowserFavourites,
  #[key("browser forward", win = BrowserForward, macos = !, linux = !)]
  BrowserForward,
  #[key("browser home", win = BrowserHome, macos = !, linux = !)]
  BrowserHome,
  #[key("browser refresh", win = BrowserRefresh, macos = !, linux = !)]
  BrowserRefresh,
  #[key("browser search", win = BrowserSearch, macos = !, linux = !)]
  BrowserSearch,
  #[key("browser stop", win = BrowserStop, macos = !, linux = !)]
  BrowserStop,
  #[key("c", win = C, macos = !, linux = !)]
  C,
  #[key("cancel", win = Cancel, macos = !, linux = !)]
  Cancel,
  #[key("capital", win = Capital, macos = !, linux = !)]
  Capital,
  #[key("clear", win = Clear, macos = !, linux = !)]
  Clear,
  #[key("control" | "ctrl" | "ctrl key" | "control key", win = Control, macos = !, linux = !)]
  Control,
  #[key("convert", win = Convert, macos = !, linux = !)]
  Convert,
  #[key("crsel", win = Crsel, macos = !, linux = !)]
  Crsel,
  #[key("d", win = D, macos = !, linux = !)]
  D,
  #[key("0", win = D0, macos = !, linux = !)]
  D0,
  #[key("1", win = D1, macos = !, linux = !)]
  D1,
  #[key("2", win = D2, macos = !, linux = !)]
  D2,
  #[key("3", win = D3, macos = !, linux = !)]
  D3,
  #[key("4", win = D4, macos = !, linux = !)]
  D4,
  #[key("5", win = D5, macos = !, linux = !)]
  D5,
  #[key("6", win = D6, macos = !, linux = !)]
  D6,
  #[key("7", win = D7, macos = !, linux = !)]
  D7,
  #[key("8", win = D8, macos = !, linux = !)]
  D8,
  #[key("9", win = D9, macos = !, linux = !)]
  D9,
  #[key("decimal", win = Decimal, macos = !, linux = !)]
  Decimal,
  #[key("delete" | "del", win = Delete, macos = !, linux = !)]
  Delete,
  #[key("divide", win = Divide, macos = !, linux = !)]
  Divide,
  #[key("down", win = Down, macos = !, linux = !)]
  Down,
  #[key("e", win = E, macos = !, linux = !)]
  E,
  #[key("end", win = End, macos = !, linux = !)]
  End,
  #[key("ereof", win = Ereof, macos = !, linux = !)]
  Ereof,
  #[key("escape" | "esc", win = Escape, macos = !, linux = !)]
  Escape,
  #[key("execute" | "exec", win = Execute, macos = !, linux = !)]
  Execute,
  #[key("exsel", win = Exsel, macos = !, linux = !)]
  Exsel,
  #[key("f", win = F, macos = !, linux = !)]
  F,
  #[key("f1", win = F1, macos = !, linux = !)]
  F1,
  #[key("f2", win = F2, macos = !, linux = !)]
  F2,
  #[key("f3", win = F3, macos = !, linux = !)]
  F3,
  #[key("f4", win = F4, macos = !, linux = !)]
  F4,
  #[key("f5", win = F5, macos = !, linux = !)]
  F5,
  #[key("f6", win = F6, macos = !, linux = !)]
  F6,
  #[key("f7", win = F7, macos = !, linux = !)]
  F7,
  #[key("f8", win = F8, macos = !, linux = !)]
  F8,
  #[key("f9", win = F9, macos = !, linux = !)]
  F9,
  #[key("f10", win = F10, macos = !, linux = !)]
  F10,
  #[key("f11", win = F11, macos = !, linux = !)]
  F11,
  #[key("f12", win = F12, macos = !, linux = !)]
  F12,
  #[key("f13", win = F13, macos = !, linux = !)]
  F13,
  #[key("f14", win = F14, macos = !, linux = !)]
  F14,
  #[key("f15", win = F15, macos = !, linux = !)]
  F15,
  #[key("f16", win = F16, macos = !, linux = !)]
  F16,
  #[key("f17", win = F17, macos = !, linux = !)]
  F17,
  #[key("f18", win = F18, macos = !, linux = !)]
  F18,
  #[key("f19", win = F19, macos = !, linux = !)]
  F19,
  #[key("f20", win = F20, macos = !, linux = !)]
  F20,
  #[key("f21", win = F21, macos = !, linux = !)]
  F21,
  #[key("f22", win = F22, macos = !, linux = !)]
  F22,
  #[key("f23", win = F23, macos = !, linux = !)]
  F23,
  #[key("f24", win = F24, macos = !, linux = !)]
  F24,
  #[key("final", win = Final, macos = !, linux = !)]
  Final,
  #[key("g", win = G, macos = !, linux = !)]
  G,
  #[key("game pad a", win = GamepadA, macos = !, linux = !)]
  GamepadA,
  #[key("game pad b", win = GamepadB, macos = !, linux = !)]
  GamepadB,
  #[key("game pad d pad down", win = GamepadDpadDown, macos = !, linux = !)]
  GamepadDpadDown,
  #[key("game pad d pad left", win = GamepadDpadLeft, macos = !, linux = !)]
  GamepadDpadLeft,
  #[key("game pad d pad right", win = GamepadDpadRight, macos = !, linux = !)]
  GamepadDpadRight,
  #[key("game pad d pad up", win = GamepadDpadUp, macos = !, linux = !)]
  GamepadDpadUp,
  #[key("game pad left shoulder", win = GamepadLeftShoulder, macos = !, linux = !)]
  GamepadLeftShoulder,
  #[key("game pad left stick" | "gamepad left thumb stick" | "gamepad left stick button" | "gamepad left thumb stick button", win = GamepadLeftThumbstickButton, macos = !, linux = !)]
  GamepadLeftThumbstickButton,
  #[key("game pad left stick down" | "gamepad left thumb stick down", win = GamepadLeftThumbstickDown, macos = !, linux = !)]
  GamepadLeftThumbstickDown,
  #[key("game pad left stick left" | "gamepad left thumb stick left", win = GamepadLeftThumbstickLeft, macos = !, linux = !)]
  GamepadLeftThumbstickLeft,
  #[key("game pad left stick right" | "gamepad left thumb stick right", win = GamepadLeftThumbstickRight, macos = !, linux = !)]
  GamepadLeftThumbstickRight,
  #[key("game pad left stick up" | "gamepad left thumb stick up", win = GamepadLeftThumbstickUp, macos = !, linux = !)]
  GamepadLeftThumbstickUp,
  #[key("game pad left trigger", win = GamepadLeftTrigger, macos = !, linux = !)]
  GamepadLeftTrigger,
  #[key("game pad right shoulder", win = GamepadRightShoulder, macos = !, linux = !)]
  GamepadRightShoulder,
  #[key("game pad menu", win = GamepadMenu, macos = !, linux = !)]
  GamepadMenu,
  #[key("game pad right stick" | "gamepad right thumb stick" | "gamepad right stick button" | "gamepad right thumb stick button", win = GamepadRightThumbstickButton, macos = !, linux = !)]
  GamepadRightThumbstickButton,
  #[key("game pad right stick down" | "gamepad right thumb stick down", win = GamepadRightThumbstickDown, macos = !, linux = !)]
  GamepadRightThumbstickDown,
  #[key("game pad right stick left" | "gamepad right thumb stick left", win = GamepadRightThumbstickLeft, macos = !, linux = !)]
  GamepadRightThumbstickLeft,
  #[key("game pad right stick right" | "gamepad right thumb stick right", win = GamepadRightThumbstickRight, macos = !, linux = !)]
  GamepadRightThumbstickRight,
  #[key("game pad right stick up" | "gamepad right thumb stick up", win = GamepadRightThumbstickUp, macos = !, linux = !)]
  GamepadRightThumbstickUp,
  #[key("game pad right trigger", win = GamepadRightTrigger, macos = !, linux = !)]
  GamepadRightTrigger,
  #[key("game pad view", win = GamepadView, macos = !, linux = !)]
  GamepadView,
  #[key("game pad x", win = GamepadX, macos = !, linux = !)]
  GamepadX,
  #[key("game pad y", win = GamepadY, macos = !, linux = !)]
  GamepadY,
  #[key("h", win = H, macos = !, linux = !)]
  H,
  #[key("help", win = Help, macos = !, linux = !)]
  Help,
  #[key("home", win = Home, macos = !, linux = !)]
  Home,
  #[key("i", win = I, macos = !, linux = !)]
  I,
  #[key("ico 00", win = Ico00, macos = !, linux = !)]
  Ico00,
  #[key("ico clear", win = IcoClear, macos = !, linux = !)]
  IcoClear,
  #[key("ico help", win = IcoHelp, macos = !, linux = !)]
  IcoHelp,
  #[key("ime off", win = ImeOff, macos = !, linux = !)]
  ImeOff,
  #[key("ime on", win = ImeOn, macos = !, linux = !)]
  ImeOn,
  #[key("insert", win = Insert, macos = !, linux = !)]
  Insert,
  #[key("j", win = J, macos = !, linux = !)]
  J,
  #[key("junja", win = Junja, macos = !, linux = !)]
  Junja,
  #[key("k", win = K, macos = !, linux = !)]
  K,
  #[key("kana", win = Kana, macos = !, linux = !)]
  Kana,
  #[key("kanji", win = Kanji, macos = !, linux = !)]
  Kanji,
  #[key("l", win = L, macos = !, linux = !)]
  L,
  #[key("launch app 1", win = LaunchApp1, macos = !, linux = !)]
  LaunchApp1,
  #[key("launch app 2", win = LaunchApp2, macos = !, linux = !)]
  LaunchApp2,
  #[key("launch mail", win = LaunchMail, macos = !, linux = !)]
  LaunchMail,
  #[key("launch media select", win = LaunchMediaSelect, macos = !, linux = !)]
  LaunchMediaSelect,
  #[key("l button" | "left button", win = LButton, macos = !, linux = !)]
  LButton,
  #[key("lcontrol" | "lctrl" | "lcontrol key" | "lctrl key" | "left control" | "left ctrl" | "left control key" | "left ctrl key", win = LControl, macos = !, linux = !)]
  LControl,
  #[key("left" | "left arrow", win = Left, macos = !, linux = !)]
  Left,
  #[key("lmenu" | "lalt" | "lalt key" | "left alt" | "left alt key", win = LMenu, macos = !, linux = !)]
  LMenu,
  #[key("lshift" | "lshift key" | "left shift" | "left shift key", win = LShift, macos = !, linux = !)]
  LShift,
  #[key("lwin" | "lwin key" | "left win" | "left win key", win = LWin, macos = !, linux = !)]
  LWin,
  #[key("m", win = M, macos = !, linux = !)]
  M,
  #[key("m button", win = MButton, macos = !, linux = !)]
  MButton,
  #[key("media next track", win = MediaNextTrack, macos = !, linux = !)]
  MediaNextTrack,
  #[key("media play pause", win = MediaPlayPause, macos = !, linux = !)]
  MediaPlayPause,
  #[key("media previous track" | "media prev track", win = MediaPrevTrack, macos = !, linux = !)]
  MediaPrevTrack,
  #[key("media stop", win = MediaStop, macos = !, linux = !)]
  MediaStop,
  #[key("menu" | "alt", win = Menu, macos = !, linux = !)]
  Menu,
  #[key("mode change", win = ModeChange, macos = !, linux = !)]
  ModeChange,
  #[key("multiply", win = Multiply, macos = !, linux = !)]
  Multiply,
  #[key("n", win = N, macos = !, linux = !)]
  N,
  #[key("navigation accept" | "nav accept", win = NavigationAccept, macos = !, linux = !)]
  NavigationAccept,
  #[key("navigation cancel" | "nav cancel", win = NavigationCancel, macos = !, linux = !)]
  NavigationCancel,
  #[key("navigation down" | "nav down", win = NavigationDown, macos = !, linux = !)]
  NavigationDown,
  #[key("navigation left" | "nav left", win = NavigationLeft, macos = !, linux = !)]
  NavigationLeft,
  #[key("navigation menu" | "nav menu", win = NavigationMenu, macos = !, linux = !)]
  NavigationMenu,
  #[key("naviagation right" | "nav right", win = NavigationRight, macos = !, linux = !)]
  NavigationRight,
  #[key("navigation up" | "nav up", win = NavigationUp, macos = !, linux = !)]
  NavigationUp,
  #[key("navigation view" | "nav view", win = NavigationView, macos = !, linux = !)]
  NavigationView,
  #[key("next", win = Next, macos = !, linux = !)]
  Next,
  #[key("no name", win = NoName, macos = !, linux = !)]
  NoName,
  #[key("non convert", win = NonConvert, macos = !, linux = !)]
  NonConvert,
  #[key("num lock", win = Numlock, macos = !, linux = !)]
  Numlock,
  #[key("num pad 0" | "number pad 0", win = Numpad0, macos = !, linux = !)]
  Numpad0,
  #[key("num pad 1" | "number pad 1", win = Numpad1, macos = !, linux = !)]
  Numpad1,
  #[key("num pad 2" | "number pad 2", win = Numpad2, macos = !, linux = !)]
  Numpad2,
  #[key("num pad 3" | "number pad 3", win = Numpad3, macos = !, linux = !)]
  Numpad3,
  #[key("num pad 4" | "number pad 4", win = Numpad4, macos = !, linux = !)]
  Numpad4,
  #[key("num pad 5" | "number pad 5", win = Numpad5, macos = !, linux = !)]
  Numpad5,
  #[key("num pad 6" | "number pad 6", win = Numpad6, macos = !, linux = !)]
  Numpad6,
  #[key("num pad 7" | "number pad 7", win = Numpad7, macos = !, linux = !)]
  Numpad7,
  #[key("num pad 8" | "number pad 8", win = Numpad8, macos = !, linux = !)]
  Numpad8,
  #[key("num pad 9" | "number pad 9", win = Numpad9, macos = !, linux = !)]
  Numpad9,
  #[key("o", win = O, macos = !, linux = !)]
  O,
  #[key("oem 1", win = Oem1, macos = !, linux = !)]
  Oem1,
  #[key("oem 102", win = Oem102, macos = !, linux = !)]
  Oem102,
  #[key("oem 2", win = Oem2, macos = !, linux = !)]
  Oem2,
  #[key("oem 3", win = Oem3, macos = !, linux = !)]
  Oem3,
  #[key("oem 4", win = Oem4, macos = !, linux = !)]
  Oem4,
  #[key("oem 5", win = Oem5, macos = !, linux = !)]
  Oem5,
  #[key("oem 6", win = Oem6, macos = !, linux = !)]
  Oem6,
  #[key("oem 7", win = Oem7, macos = !, linux = !)]
  Oem7,
  #[key("oem 8", win = Oem8, macos = !, linux = !)]
  Oem8,
  #[key("oem attn", win = OemAttn, macos = !, linux = !)]
  OemAttn,
  #[key("oem auto", win = OemAuto, macos = !, linux = !)]
  OemAuto,
  #[key("oem ax", win = OemAx, macos = !, linux = !)]
  OemAx,
  #[key("oem back tab", win = OemBacktab, macos = !, linux = !)]
  OemBacktab,
  #[key("oem clear", win = OemClear, macos = !, linux = !)]
  OemClear,
  #[key("oem comma", win = OemComma, macos = !, linux = !)]
  OemComma,
  #[key("oem copy", win = OemCopy, macos = !, linux = !)]
  OemCopy,
  #[key("oem cusel", win = OemCusel, macos = !, linux = !)]
  OemCusel,
  #[key("oem enlw", win = OemEnlw, macos = !, linux = !)]
  OemEnlw,
  #[key("oem finish", win = OemFinish, macos = !, linux = !)]
  OemFinish,
  #[key("oem fj loya", win = OemFjLoya, macos = !, linux = !)]
  OemFjLoya,
  #[key("oem fj masshou", win = OemFjMasshou, macos = !, linux = !)]
  OemFjMasshou,
  #[key("oem fj roya", win = OemFjRoya, macos = !, linux = !)]
  OemFjRoya,
  #[key("oem fj touroku", win = OemFjTouroku, macos = !, linux = !)]
  OemFjTouroku,
  #[key("oem jump", win = OemJump, macos = !, linux = !)]
  OemJump,
  #[key("oem minus", win = OemMinus, macos = !, linux = !)]
  OemMinus,
  #[key("oem nec equal", win = OemNecEqual, macos = !, linux = !)]
  OemNecEqual,
  #[key("oem pa 1", win = OemPa1, macos = !, linux = !)]
  OemPa1,
  #[key("oem pa 2", win = OemPa2, macos = !, linux = !)]
  OemPa2,
  #[key("oem pa 3", win = OemPa3, macos = !, linux = !)]
  OemPa3,
  #[key("oem period" | "oem dot", win = OemPeriod, macos = !, linux = !)]
  OemPeriod,
  #[key("oem plus", win = OemPlus, macos = !, linux = !)]
  OemPlus,
  #[key("oem reset", win = OemReset, macos = !, linux = !)]
  OemReset,
  #[key("oem ws ctrl" | "oem ws control", win = OemWsCtrl, macos = !, linux = !)]
  OemWsCtrl,
  #[key("p", win = P, macos = !, linux = !)]
  P,
  #[key("pa 1", win = PA1, macos = !, linux = !)]
  PA1,
  #[key("packet", win = Packet, macos = !, linux = !)]
  Packet,
  #[key("pause" | "pause break" | "break", win = Pause, macos = !, linux = !)]
  Pause,
  #[key("play", win = Play, macos = !, linux = !)]
  Play,
  #[key("print", win = Print, macos = !, linux = !)]
  Print,
  #[key("prior", win = Prior, macos = !, linux = !)]
  Prior,
  #[key("process" | "process key", win = Processkey, macos = !, linux = !)]
  Processkey,
  #[key("q", win = Q, macos = !, linux = !)]
  Q,
  #[key("r", win = R, macos = !, linux = !)]
  R,
  #[key("r button", win = Rbutton, macos = !, linux = !)]
  RButton,
  #[key("rctrl" | "rcontrol" | "rctrl key" | "r control key" | "right ctrl" | "right control" | "right ctrl key" | "right control key", win = RControl, macos = !, linux = !)]
  RControl,
  #[key("return" | "enter", win = Return, macos = !, linux = !)]
  Return,
  #[key("right" | "right arrow", win = Right, macos = !, linux = !)]
  Right,
  #[key("rmenu" | "ralt" | "right alt" | "right menu", win = RMenu, macos = !, linux = !)]
  RMenu,
  #[key("rshift" | "right shift" | "rshift key" | "right shift key", win = RShift, macos = !, linux = !)]
  RShift,
  #[key("rwin" | "right win" | "rwin key" | "right win key", win = RWin, macos = !, linux = !)]
  RWin,
  #[key("s", win = S, macos = !, linux = !)]
  S,
  #[key("scroll", win = Scroll, macos = !, linux = !)]
  Scroll,
  #[key("select", win = Select, macos = !, linux = !)]
  Select,
  #[key("separator", win = Separator, macos = !, linux = !)]
  Separator,
  #[key("shift" | "shift key", win = Shift, macos = !, linux = !)]
  Shift,
  #[key("sleep", win = Sleep, macos = !, linux = !)]
  Sleep,
  #[key("snapshot", win = Snapshot, macos = !, linux = !)]
  Snapshot,
  #[key("space", win = Space, macos = !, linux = !)]
  Space,
  #[key("subtract", win = Subtract, macos = !, linux = !)]
  Subtract,
  #[key("t", win = T, macos = !, linux = !)]
  T,
  #[key("tab", win = Tab, macos = !, linux = !)]
  Tab,
  #[key("u", win = U, macos = !, linux = !)]
  U,
  #[key("up" | "up arrow", win = Up, macos = !, linux = !)]
  Up,
  #[key("v", win = V, macos = !, linux = !)]
  V,
  #[key("volume down", win = VolumeDown, macos = !, linux = !)]
  VolumeDown,
  #[key("volume mute" | "mute", win = VolumeMute, macos = !, linux = !)]
  VolumeMute,
  #[key("volume up", win = VolumeUp, macos = !, linux = !)]
  VolumeUp,
  #[key("w", win = W, macos = !, linux = !)]
  W,
  #[key("win" | "win key", win = Virt(LWin), macos = !, linux = !)]
  Win,
  #[key("x", win = X, macos = !, linux = !)]
  X,
  #[key("x button 1", win = XButton1, macos = !, linux = !)]
  XButton1,
  #[key("x button 2", win = XButton2, macos = !, linux = !)]
  XButton2,
  #[key("y", win = Y, macos = !, linux = !)]
  Y,
  #[key("z", win = Z, macos = !, linux = !)]
  Z,
  #[key("zoom", win = Zoom, macos = !, linux = !)]
  Zoom,
  #[key("none", win = None, macos = !, linux = !)]
  None,

  #[key(..)]
  Custom(u16),
}

impl Key {
  /// Check if the key is analogous to another key.
  /// Returns `true` if the keys are the same or if `other` is a
  /// specialisation of self.
  ///
  /// # Example
  /// ```
  /// assert!(Key::A.is_analogous(Key::A));
  /// assert!(Key::Shift.is_analogous(Key::LShift));
  /// assert!(Key::LControl.is_analogous(Key::RControl) == false);
  /// ```
  pub fn is_analogous(self, other: Key) -> bool {
    #[allow(clippy::match_same_arms)]
    match (self, other) {
      (Key::Shift, Key::LShift | Key::RShift) => true,
      (Key::Control, Key::LControl | Key::RControl) => true,
      (Key::Menu, Key::LMenu | Key::RMenu) => true,
      (Key::Win, Key::LWin | Key::RWin) => true,
      _ => self == other,
    }
  }

  /// Returns whether the key is a generic key, such as `Shift`, `Control`,
  /// `Alt`, or `Win` instead of the more specific versions like
  /// `LShift`, `RShift`, etc.
  pub fn is_generic(self) -> bool {
    matches!(self, Key::Shift | Key::Control | Key::Menu | Key::Win)
  }

  /// Returns a generic version of the key.
  /// Special keys like `LShift`, `RShift`, `LControl`, `RControl` etc.
  /// will return the gereric Shift, Control, Alt, or Win key.
  /// Other keys will return themselves unchanged.
  pub fn get_generic(self) -> Self {
    match self {
      Key::LShift | Key::RShift => Key::Shift,
      Key::LControl | Key::RControl => Key::Control,
      Key::LMenu | Key::RMenu => Key::Menu,
      Key::LWin | Key::RWin => Key::Win,
      _ => self,
    }
  }

  /// Get the specific key(s) for a Key.
  /// Generic keys like `Shift`, `Control`, `Alt`, and `Win` will return
  /// both the left and right versions of the key.
  /// Non-generic keys will return a vector containing just the key itself.
  pub fn get_specifics(self) -> Vec<Key> {
    match self {
      Key::Shift => vec![Key::LShift, Key::RShift],
      Key::Control => vec![Key::LControl, Key::RControl],
      Key::Menu => vec![Key::LMenu, Key::RMenu],
      Key::Win => vec![Key::LWin, Key::RWin],
      _ => vec![self],
    }
  }

  /// Gets whether this key is currently down.
  pub fn is_down(self) -> bool {
    self
      .get_specifics()
      .iter()
      .any(|key| key.into_vk().is_down_raw())
  }
}
