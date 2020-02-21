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

            if (parent is SplitContainer == false)
                return null;

            if (currentChildren.Count == 0)
            {
                // TODO: Take up full width with the container to attach.
                // This if-statement might not be necessary (instead handle below).
                command.Parent.AddChild(command.NewChild);
            }

            var innerGaps = _userConfigService.UserConfig.InnerGap;

            if (parent.Orientation == "horizontal")
            {
                var newChildWidth = (currentChildren.Count + 1) / (parent.Width - (innerGaps * currentChildren.Count));
                var newChildHeight = parent.Height;
            }

            if (parent.Orientation == "vertical")
            {
                var newChildWidth = parent.Width;
                var newChildHeight = (currentChildren.Count + 1) / (parent.Width - (innerGaps * currentChildren.Count));
            }

            return CommandResponse.Ok;
        }
    }
}
