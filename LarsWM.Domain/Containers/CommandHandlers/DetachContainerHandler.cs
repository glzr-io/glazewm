using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
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
            var currentChildren = command.Parent.Children;

            var innerGap = _userConfigService.UserConfig.InnerGap;

            if (parent.Layout == Layout.Horizontal)
            {
                // Available parent width is the width of the parent minus all inner gaps.
                var currentAvailableParentWidth = parent.Width - (innerGap * (currentChildren.Count - 1));
                var newAvailableParentWidth = parent.Width - (innerGap * (currentChildren.Count - 2));

                // Adjust widths of current child containers.
                foreach (var currentChild in currentChildren)
                {
                    // TODO: Width percentage is incorrect.
                    // Eg. if 33%/33%/33% -> 50%/50% or 50%/25%/25% -> 62.5%/37.5%
                    // Get the percentage of the child to remove and distribute it evenly amongst new children.
                    var widthPercentage = (double)currentChild.Width / currentAvailableParentWidth;
                    currentChild.Width = (int)(widthPercentage * (newAvailableParentWidth + childToRemove.Width));
                }

                parent.RemoveChild(childToRemove);

                // Adjust x-coordinate of child containers.
                Container previousChild = null;
                foreach (var child in parent.Children)
                {
                    if (previousChild == null)
                        child.X = parent.X;

                    else
                        child.X = previousChild.X + previousChild.Width + innerGap;

                    previousChild = child;
                }
            }

            if (parent.Layout == Layout.Vertical)
            {
            }

            _containerService.PendingContainersToRedraw.Add(parent);

            return CommandResponse.Ok;
        }
    }
}
