using System;
using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Containers.CommandHandlers
{
    class RedrawContainersHandler : ICommandHandler<RedrawContainersCommand>
    {
        private IBus _bus;
        private ContainerService _containerService;
        private readonly UserConfigService _userConfigService;

        public RedrawContainersHandler(IBus bus, ContainerService containerService, UserConfigService userConfigService)
        {
            _bus = bus;
            _containerService = containerService;
            _userConfigService = userConfigService;
        }

        public dynamic Handle(RedrawContainersCommand command)
        {
            var containersToRedraw = _containerService.PendingContainersToRedraw;

            var uniqueContainersToRedraw = containersToRedraw.SelectMany(container => container.Flatten()).Distinct();

            var handle = BeginDeferWindowPos(uniqueContainersToRedraw.Count());

            foreach (var container in uniqueContainersToRedraw)
            {
                var flags = SWP.SWP_FRAMECHANGED | SWP.SWP_NOACTIVATE | SWP.SWP_NOCOPYBITS |
                    SWP.SWP_NOZORDER | SWP.SWP_NOOWNERZORDER;

                DeferWindowPos(handle, window.Hwnd, IntPtr.Zero, container.X, container.Y, container.Width, container.Height, flags);
            }

            EndDeferWindowPos(handle);

            return CommandResponse.Ok;
        }
    }
}
