using System.Threading;
using System.Windows.Forms;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class InspectWindowHandler : ICommandHandler<InspectWindowCommand>
  {
    public CommandResponse Handle(InspectWindowCommand command)
    {
      // open inspector on a separate thread
      var thread = new Thread(() =>
      {
        using var inspector = new Inspector();
        inspector.ShowDialog(GetParentWindow(command.WindowToInspect));
      });

      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();

      return CommandResponse.Ok;
    }

    private static NativeWindow GetParentWindow(Window windowToInspect)
    {
      if (windowToInspect == null)
      {
        return null;
      }

      var parentWindow = new NativeWindow();
      parentWindow.AssignHandle(windowToInspect.Handle);

      return parentWindow;
    }
  }
}
