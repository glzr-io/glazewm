using System;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class ResizeFocusedWindowHandler : ICommandHandler<ResizeFocusedWindowCommand>
    {
        private WindowService _windowService;

        public ResizeFocusedWindowHandler(WindowService windowService)
        {
            _windowService = windowService;
        }

        public dynamic Handle(ResizeFocusedWindowCommand command)
        {
            throw new NotImplementedException();
        }
    }
}
