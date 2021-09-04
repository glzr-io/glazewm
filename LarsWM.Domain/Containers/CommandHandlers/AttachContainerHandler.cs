using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Windows;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private MonitorService _monitorService;

    public AttachContainerHandler(Bus bus, ContainerService containerService, MonitorService monitorService)
    {
      _bus = bus;
      _containerService = containerService;
      _monitorService = monitorService;
    }

    public dynamic Handle(AttachContainerCommand command)
    {
      var parent = command.Parent;
      var childToAdd = command.ChildToAdd;

      if (childToAdd.Parent != null)
      {
        var currentMonitor = _monitorService.GetMonitorFromChildContainer(childToAdd);
        var newMonitor = _monitorService.GetMonitorFromChildContainer(parent);

        if (currentMonitor.Dpi != newMonitor.Dpi && childToAdd is Window)
        {
          var dpiScaleFactor = decimal.Divide(currentMonitor.Dpi, newMonitor.Dpi);
          (childToAdd as Window).PendingDpiScaling = dpiScaleFactor;
        }

        _bus.Invoke(new DetachContainerCommand(childToAdd.Parent as SplitContainer, childToAdd));
      }

      parent.Children.Insert(command.InsertPosition, childToAdd);
      childToAdd.Parent = parent;

      // Adjust SizePercentage of self and siblings.
      double defaultPercent = 1.0 / parent.Children.Count;
      foreach (var child in parent.Children)
        child.SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(parent);

      // Adjust focus order of ancestors if the attached container is focused.
      if (_containerService.FocusedContainer == childToAdd)
        _bus.Invoke(new SetFocusedDescendantCommand(childToAdd));

      return CommandResponse.Ok;
    }
  }
}
