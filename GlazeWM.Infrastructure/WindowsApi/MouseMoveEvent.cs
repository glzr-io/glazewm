using System;
using System.Runtime.InteropServices;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.WindowsApi;

public record MouseMoveEvent(
  /// <summary>
  /// (X,Y) location of mouse with 0,0 being the top-left corner of the main monitor
  /// </summary>
  Point Point,

  /// <summary>
  /// Whether left-click is currently pressed.
  /// </summary>
  bool IsMouseDown,

  /// <summary>
  /// The time stamp for this message, equivalent to what `GetMessageTime` would
  /// return.
  /// </summary>
  int TimeStamp
) : Event(InfraEvent.MouseMove);

public enum WMessages
{
  WM_MOUSEMOVE = 0x0200,
  WM_LBUTTONDOWN = 0x201,
  WM_LBUTTONUP = 0x202,
  WM_RBUTTONDOWN = 0x0204,
  WM_RBUTTONUP = 0x0205,
  WM_KEYDOWN = 0x100,
  WM_KEYUP = 0x101,
  WH_KEYBOARD_LL = 13,
  WH_MOUSE_LL = 14,
}
[StructLayout(LayoutKind.Sequential)]
public struct LowLevelMouseInputEvent
{
  /// <summary>
  /// Mouse event details.
  /// </summary>
  public LowLevelMouseInputEventDetails lParam;

  /// <summary>
  /// Determines how to process the event.
  /// </summary>
  public int nCode;

  /// <summary>
  /// Identify type of event.
  /// </summary>
  public WMessages wParam;
}

[StructLayout(LayoutKind.Sequential)]
public struct LowLevelMouseInputEventDetails
{
  /// <summary>
  /// (X,Y) location of mouse with 0,0 being the top-left corner of the main monitor
  /// </summary>
  public Point pt;

  /// <summary>
  /// Mouse data.
  /// </summary>
  public int mouseData;

  /// <summary>
  /// The extended-key flag, event-injected Flags, context code, and transition-state flag. This member is specified as follows. An application can use the following values to test the keystroke Flags. Testing LLKHF_INJECTED (bit 4) will tell you whether the event was injected. If it was, then testing LLKHF_LOWER_IL_INJECTED (bit 1) will tell you whether or not the event was injected from a process running at lower integrity level.
  /// </summary>
  public int Flags;

  /// <summary>
  /// The time stamp for this message, equivalent to what GetMessageTime would return for this message.
  /// </summary>
  public int TimeStamp;

  /// <summary>
  /// Additional information associated with the message.
  /// </summary>
  public IntPtr dwExtraInfo;
}
