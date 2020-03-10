using System.Linq;
using LarsWM.Domain.Common.Enums;
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

            parent.RemoveChild(childToRemove);

            // Siblings of the removed child.
            var siblings = command.Parent.Children;

            var isEmptySplitContainer = siblings.Count() == 0 && !(parent is Workspace);

            double defaultPercent = 1.0 / siblings.Count;

            // If the parent of the removed child is an empty split container, remove
            // the split container as well.
            if (isEmptySplitContainer)
            {
                var grandparent = parent.Parent;
                grandparent.RemoveChild(parent);

                // TODO: Perhaps create a private method that takes the container with children
                // to adjust that has the SizePercentage and default percent logic. Alternatively
                // create a variable containerToAdjust that is then operated on.
                foreach (var child in grandparent.Children)
                    child.SizePercentage = defaultPercent;

                // TODO: Fix issue where grandparent is somehow a split container after vertical ->
                // -> horizontal split.
                _containerService.SplitContainersToRedraw.Add(grandparent as SplitContainer);

                return CommandResponse.Ok;
            }

            // TODO: Adjust SizePercentage of children based on their previous SizePercentage.

            foreach (var child in siblings)
                child.SizePercentage = defaultPercent;

            _containerService.SplitContainersToRedraw.Add(parent);

            return CommandResponse.Ok;
        }
    }
}
