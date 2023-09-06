using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class InspectWindowHandler : ICommandHandler<InspectWindowCommand>
  {
    public CommandResponse Handle(InspectWindowCommand command)
    {
      var inspector = new Inspector();
      inspector.SetTitle(command.WindowToInspect.Title);
      inspector.SetClassName(command.WindowToInspect.ClassName);
      inspector.SetProcessName(command.WindowToInspect.ProcessName);
      inspector.StartPosition = System.Windows.Forms.FormStartPosition.CenterParent;

      // show dialog as child of parent window
      var parentWindow = new System.Windows.Forms.NativeWindow();
      parentWindow.AssignHandle(command.WindowToInspect.Handle);
      inspector.ShowDialog(parentWindow);

      return CommandResponse.Ok;
    }
  }
}
