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

        // Use for attaching windows to workspaces/split containers
        // Consider using it for attaching workspaces to monitors as well.
        public dynamic Handle(AttachContainerCommand command)
        {
            var parent = command.Parent;
            var newChild = command.NewChild;
            var currentChildren = command.Parent.Children;

            var innerGaps = _userConfigService.UserConfig.InnerGap;

            if (parent.Layout == Layout.Horizontal)
            {
                // Direct children of parent have the same height as parent in horizontal layouts.
                newChild.Height = parent.Height;

                // Available parent width is the width of the parent minus all inner gaps.
                var currentAvailableParentWidth = parent.Width - (innerGaps * currentChildren.Count - 1);
                var newAvailableParentWidth = parent.Width - (innerGaps * currentChildren.Count);

                newChild.Width = (currentChildren.Count + 1) / newAvailableParentWidth;

                // Adjust widths of current child containers.
                foreach (var currentChild in currentChildren)
                {
                    var widthPercentage = (currentChild.Width / currentAvailableParentWidth) * 100;
                    currentChild.Width = widthPercentage * (newAvailableParentWidth - newChild.Width);
                }

                parent.Children.Add(newChild);
                newChild.Parent = parent;

                Container previousChild = null;

                // Adjust x-coordinate of child containers.
                foreach (var child in parent.Children)
                {
                    if (child == parent.Children[0])
                    {
                        previousChild = child;
                        continue;
                    }

                    child.X = previousChild.X + previousChild.Width + innerGaps;

                    previousChild = child;
                }
            }

            if (parent.Layout == Layout.Vertical)
            {
                var newChildWidth = parent.Width;
                var newChildHeight = (currentChildren.Count + 1) / (parent.Width - (innerGaps * currentChildren.Count));
            }

            _containerService.PendingContainersToRedraw.Add(parent);

            return CommandResponse.Ok;
        }
    }
}
