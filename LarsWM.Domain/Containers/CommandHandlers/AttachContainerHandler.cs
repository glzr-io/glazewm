using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.Monitors.Events;
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
                var newChildHeight = parent.Height;

                var availableParentWidth = parent.Width - (innerGaps * currentChildren.Count);
                var newChildWidth = (currentChildren.Count + 1) / availableParentWidth;

                var availableParentWidthWithChild = availableParentWidth - newChildWidth;

                // Adjust widths of current child containers.
                foreach (var currentChild in currentChildren)
                {
                    var widthPercentage = (currentChild.Width / availableParentWidth) * 100;
                    currentChild.Width = widthPercentage * availableParentWidthWithChild;
                }

                parent.Children.Add(newChild);
                newChild.Parent = parent;
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
