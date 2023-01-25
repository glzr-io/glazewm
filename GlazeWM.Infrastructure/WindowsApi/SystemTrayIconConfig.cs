using System;
using System.Collections.Generic;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class NetworkIconConfig
  {
    public string HoverText { get; init; }
    public string IconResourceName { get; init; }
    public Dictionary<string, Action> Actions { get; init; }
  }
}
