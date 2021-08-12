using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class AttachContainerHandler : ICommandHandler<AttachContainerCommand>
  {
    private IBus _bus;
    private ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public AttachContainerHandler(IBus bus, ContainerService containerService, UserConfigService userConfigService)
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

      // TODO: Adjust SizePercentage of current children.

      parent.AddChild(newChild);

      double defaultPercent = 1.0 / children.Count;
      foreach (var child in children)
        child.SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(parent);

      return CommandResponse.Ok;
    }
  }
}
