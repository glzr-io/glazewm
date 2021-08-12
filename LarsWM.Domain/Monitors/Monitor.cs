using LarsWM.Domain.Containers;
using LarsWM.Domain.Workspaces;
using System;
using System.Collections.Generic;
using System.Windows.Forms;

namespace LarsWM.Domain.Monitors
{
  public class Monitor : Container
  {
    public Guid Id = Guid.NewGuid();
    public string Name => Screen.DeviceName;
    public override int Width => Screen.WorkingArea.Width;
    public override int Height => Screen.WorkingArea.Height;
    public override int X => Screen.WorkingArea.X;
    public override int Y => Screen.WorkingArea.Y;
    public bool IsPrimary => Screen.Primary;
    public Workspace DisplayedWorkspace;  // Alternatively add IsDisplayed/IsVisible property to Workspace instance

    public Screen Screen { get; }

    public Monitor(Screen screen)
    {
      Screen = screen;
    }
  }
}
