using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
    class DetachContainerHandler : ICommandHandler<DetachContainerCommand>
    {
        private IBus _bus;
        private ContainerService _containerService;
        private readonly UserConfigService _userConfigService;

        public DetachContainerHandler(IBus bus, ContainerService containerService, UserConfigService userConfigService)
        {
            _bus = bus;
            _containerService = containerService;
            _userConfigService = userConfigService;
        }

        public dynamic Handle(DetachContainerCommand command)
        {
            var parent = command.Parent;
            var childToRemove = command.ChildToRemove;
            var children = command.Parent.Children;

            parent.RemoveChild(childToRemove);

            var siblings = children.Where(child => child != childToRemove);
            var isEmptySplitContainer = siblings.Count() == 0 && !(parent is Workspace);

            if (isEmptySplitContainer)
            {
                var grandparent = parent.Parent;

                grandparent.RemoveChild(parent);
                _containerService.SplitContainersToRedraw.Add(grandparent as SplitContainer);

                return CommandResponse.Ok;
            }

            double defaultPercent = 1.0 / children.Count;

            // TODO: Adjust SizePercentage of children based on their previous SizePercentage.

            foreach (var child in children)
                child.SizePercentage = defaultPercent;

            _containerService.SplitContainersToRedraw.Add(parent);

            return CommandResponse.Ok;
        }
    }
}
