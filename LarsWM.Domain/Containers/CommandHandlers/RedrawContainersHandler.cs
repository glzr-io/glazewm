using System;
using System.Collections.Generic;
using System.Linq;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Containers.CommandHandlers
{
    class RedrawContainersHandler : ICommandHandler<RedrawContainersCommand>
    {
        private ContainerService _containerService;
        private UserConfigService _userConfigService;

        public RedrawContainersHandler(ContainerService containerService, UserConfigService userConfigService)
        {
            _containerService = containerService;
            _userConfigService = userConfigService;
        }

        public dynamic Handle(RedrawContainersCommand command)
        {
            var containersToRedraw = _containerService.SplitContainersToRedraw;

            // Recursively adjust children of SplitContainers. If another SplitContainer is
            // encountered, perform same function on its children.

            var innerGap = _userConfigService.UserConfig.InnerGap;

            // TODO: Instead of looping over containersToRedraw, create an enumerable of all SplitContainers within containersToRedraw.
            foreach (var parentContainer in containersToRedraw)
            {
                var children = parentContainer.Children;

                if (parentContainer.Layout == Common.Enums.Layout.Horizontal)
                {
                    // Available parent width is the width of the parent minus all inner gaps.
                    var availableParentWidth = parentContainer.Width - (innerGap * (children.Count - 1));

                    // Adjust widths of current child containers.
                    foreach (var child in children)
                    {
                        child.Width = (int)(child.SizePercentage * availableParentWidth);
                    }

                    // Adjust x-coordinate of child containers.
                    Container previousChild = null;
                    foreach (var child in parentContainer.Children)
                    {
                        if (previousChild == null)
                            child.X = parentContainer.X;

                        else
                            child.X = previousChild.X + previousChild.Width + innerGap;

                        previousChild = child;
                    }
                }
            }

            PushUpdates(containersToRedraw);

            return CommandResponse.Ok;
        }

        private void PushUpdates(List<SplitContainer> containersToRedraw)
        {
            var windowsToRedraw = containersToRedraw
                .SelectMany(container => container.Flatten())
                .OfType<Window>()
                .Distinct()
                .ToList();

            var handle = BeginDeferWindowPos(windowsToRedraw.Count());

            foreach (var window in windowsToRedraw)
            {
                var flags = SWP.SWP_FRAMECHANGED | SWP.SWP_NOACTIVATE | SWP.SWP_NOCOPYBITS |
                    SWP.SWP_NOZORDER | SWP.SWP_NOOWNERZORDER;

                if (window.IsHidden)
                    flags |= SWP.SWP_HIDEWINDOW;
                else
                    flags |= SWP.SWP_SHOWWINDOW;

                DeferWindowPos(handle, window.Hwnd, IntPtr.Zero, window.X, window.Y, window.Width, window.Height, flags);
            }

            EndDeferWindowPos(handle);

            containersToRedraw.Clear();
        }
    }
}
