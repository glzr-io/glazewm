using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class FocusWindowHandler : ICommandHandler<FocusWindowCommand>
    {
        private WindowService _windowService;

        public FocusWindowHandler(WindowService windowService)
        {
            _windowService = windowService;
        }

        public dynamic Handle(FocusWindowCommand command)
        {
            var window = command.Window;

            if (window == _windowService.FocusedWindow)
                return CommandResponse.Ok;

            _windowService.FocusedWindow = window;

            // Traverse upwards, creating a focus stack towards the newly focused window.
            var parent = window.Parent;
            while (parent != null)
            {
                parent.LastFocusedContainer = window;
                parent = parent.Parent;
            }

            SetForegroundWindow(window.Hwnd);

            return new CommandResponse(true, window.Id);
        }
    }
}
