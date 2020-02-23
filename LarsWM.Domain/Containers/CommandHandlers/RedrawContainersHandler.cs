using System;
using System.Collections.Generic;
using System.Linq;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Containers.CommandHandlers
{
    class RedrawContainersHandler : ICommandHandler<RedrawContainersCommand>
    {
        private ContainerService _containerService;

        public RedrawContainersHandler(ContainerService containerService)
        {
            _containerService = containerService;
        }

        public dynamic Handle(RedrawContainersCommand command)
        {
            var containersToRedraw = _containerService.PendingContainersToRedraw;

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

                DeferWindowPos(handle, window.Hwnd, IntPtr.Zero, window.X, window.Y, window.Width, window.Height, flags);
            }

            EndDeferWindowPos(handle);

            return CommandResponse.Ok;
        }
    }
}
