using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public AttachContainerHandler(Bus bus, ContainerService containerService, UserConfigService userConfigService)
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    public dynamic Handle(AttachContainerCommand command)
    {
      var parent = command.Parent;
      var newChild = command.NewChild;
      var children = command.Parent.Children;

      if (newChild.Parent != null)
        _bus.Invoke(new DetachContainerCommand(newChild.Parent as SplitContainer, newChild));

      // TODO: Adjust SizePercentage of current children.
      if (command.InsertPosition == InsertPosition.END)
        parent.AddChild(newChild);
      else
      {
        parent.Children.Insert(0, newChild);
        newChild.Parent = parent;
      }

      double defaultPercent = 1.0 / children.Count;
      foreach (var child in children)
        child.SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
