using System;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
    class ChangeContainerLayoutHandler : ICommandHandler<ChangeContainerLayout>
    {
        private IBus _bus;
        private ContainerService _containerService;
        private readonly UserConfigService _userConfigService;

        public ChangeContainerLayoutHandler(IBus bus, ContainerService containerService, UserConfigService userConfigService)
        {
            _bus = bus;
            _containerService = containerService;
            _userConfigService = userConfigService;
        }

        public dynamic Handle(ChangeContainerLayout command)
        {
            throw new NotImplementedException();
        }
    }
}
