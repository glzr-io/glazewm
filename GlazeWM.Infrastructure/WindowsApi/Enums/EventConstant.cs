namespace GlazeWM.Infrastructure.WindowsApi.Enums
{
  /// <summary>
  /// Only the subset of event constants relevant to this application are included.
  /// </summary>
  public enum EventConstant : uint
  {
    Foreground = 0x0003,
    MoveSizeEnd = 0x000B,
    MinimizeStart = 0x0016,
    MinimizeEnd = 0x0017,
    Destroy = 0x8001,
    Show = 0x8002,
    Hide = 0x8003,
    LocationChange = 0x800B,
    NameChange = 0x800C,
    ObjectCloaked = 0x8017,
    ObjectUncloaked = 0x8018
  }
}
