using System;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
    class ChangeContainerLayoutHandler : ICommandHandler<ChangeContainerLayoutCommand>
    {
        private IBus _bus;
        private ContainerService _containerService;
        private WindowService _windowService;

        public ChangeContainerLayoutHandler(IBus bus, ContainerService containerService, WindowService windowService)
        {
            _bus = bus;
            _containerService = containerService;
            _windowService = windowService;
        }

        public dynamic Handle(ChangeContainerLayoutCommand command)
        {
            var focusedWindow = _windowService.FocusedWindow;
            var parent = focusedWindow.Parent as SplitContainer;

            var newLayout = command.NewLayout;
            var currentLayout = parent.Layout;

            if (currentLayout == newLayout)
                return CommandResponse.Ok;

            if (newLayout == Layout.Vertical)
            {
                _bus.Invoke(new DetachContainerCommand(parent, focusedWindow));

                var newParent = new SplitContainer
                {
                    Layout = Layout.Vertical,
                    SizePercentage = 1
                };

                _bus.Invoke(new AttachContainerCommand(parent, newParent));
                _bus.Invoke(new AttachContainerCommand(newParent, focusedWindow));
            }

            return CommandResponse.Ok;
        }
    }
}
