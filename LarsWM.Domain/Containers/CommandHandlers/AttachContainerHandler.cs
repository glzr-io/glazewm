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

            var innerGap = _userConfigService.UserConfig.InnerGap;

            if (parent.Layout == Layout.Horizontal)
            {
                // Direct children of parent have the same height and Y coord as parent in horizontal layouts.
                newChild.Height = parent.Height;
                newChild.Y = parent.Y;

                // Available parent width is the width of the parent minus all inner gaps.
                var currentAvailableParentWidth = parent.Width - (innerGap * (currentChildren.Count - 1));
                var newAvailableParentWidth = parent.Width - (innerGap * currentChildren.Count);

                newChild.Width = newAvailableParentWidth / (currentChildren.Count + 1);

                // Adjust widths of current child containers.
                foreach (var currentChild in currentChildren)
                {
                    var widthPercentage = (double)currentChild.Width / currentAvailableParentWidth;
                    currentChild.Width = (int)(widthPercentage * (newAvailableParentWidth - newChild.Width));
                }

                parent.Children.Add(newChild);
                newChild.Parent = parent;

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
                var newChildWidth = parent.Width;
                var newChildHeight = (currentChildren.Count + 1) / (parent.Width - (innerGap * currentChildren.Count));
            }

            _containerService.PendingContainersToRedraw.Add(parent);

            return CommandResponse.Ok;
        }
    }
}
