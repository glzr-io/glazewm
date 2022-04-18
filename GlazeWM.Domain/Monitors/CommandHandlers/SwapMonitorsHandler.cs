using System;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.CommandHandlers
{
  public class SwapMonitorsHandler : ICommandHandler<SwapMonitorsCommand>
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;

    public SwapMonitorsHandler(Bus bus, MonitorService monitorService)
    {
      _bus = bus;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(SwapMonitorsCommand command)
    {
      var monitors = _monitorService.GetMonitors();

      var displayedWorkspaces = MoveWorkpaces(monitors);
      
      foreach(var workspace in displayedWorkspaces)
      {
        _bus.Invoke(new DisplayWorkspaceCommand(workspace));
      }
      
      return CommandResponse.Ok;
    }

    private List<Workspace> MoveWorkpaces(IEnumerable<Monitor> monitors)
    {
      List<Workspace> displayedWorkspaces = new List<Workspace>();
      List<Workspace> firstMonitorsChildren = monitors.First().Children.Cast<Workspace>().ToList();
      for (int i = 0; i < monitors.Count(); i++)
      {
        Monitor monitor = monitors.ElementAt(i);

        List<Workspace> nextMonitorsChildren;
        int next = i + 1;
        if (next == monitors.Count())
        {
          nextMonitorsChildren = firstMonitorsChildren;
        }
        else
        {
          nextMonitorsChildren = monitors.ElementAt(next).Children.Cast<Workspace>().ToList();
        }

        displayedWorkspaces.Add(monitor.DisplayedWorkspace);

        foreach (var workspace in nextMonitorsChildren)
        {
          _bus.Invoke(new MoveContainerWithinTreeCommand(workspace, monitor, false));
          _bus.RaiseEvent(new WorkspaceActivatedEvent(workspace));
        }
      }
      return displayedWorkspaces;
    }
  }
}
