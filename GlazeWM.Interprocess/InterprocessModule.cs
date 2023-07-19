using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using Qmmands;

namespace GlazeWM.Interprocess
{
  internal abstract class InterprocessModule : ModuleBase<InterprocessContext>
  {
    public ContainerService ContainerService { get; set; }

    public WorkspaceService WorkspaceService { get; set; }

    public WindowService WindowService { get; set; }

    public MonitorService MonitorService { get; set; }

    protected Guid SessionId
      => Context.Message.SessionId;
  }
}
