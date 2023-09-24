using System;
using System.Reactive.Linq;
using System.Windows.Forms;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public partial class Inspector : Form
  {
    private Point? cursorPosition { get; set; }
    private IDisposable cursorSubscription { get; set; }

    public Inspector()
    {
      InitializeComponent();
      InitializeCursorSubscription();

      MaximizeBox = false;
      MinimizeBox = false;
      FormBorderStyle = FormBorderStyle.FixedSingle;
      StartPosition = FormStartPosition.CenterParent;
    }

    private void InitializeCursorSubscription()
    {
      cursorSubscription = MouseEvents.MouseMoves
        .Sample(TimeSpan.FromMilliseconds(50))
        .Subscribe((@event) =>
        {
          // skip if the cursor hasn't moved
          if (cursorPosition?.X == @event.Point.X && cursorPosition?.Y == @event.Point.Y)
          {
            return;
          }

          // update last known cursor position
          cursorPosition = @event.Point;

          // update the inspector info
          UpdateInspectorValues(WindowFromPoint(cursorPosition.Value));
        });
    }

    public void UpdateInspectorValues(IntPtr? handle)
    {
      if (handle == null)
      {
        return;
      }

      // get handle details
      var processName = WindowService.GetProcessOfHandle(handle.Value)?.ProcessName ?? string.Empty;
      var title = WindowService.GetTitleOfHandle(handle.Value) ?? string.Empty;
      var className = WindowService.GetClassNameOfHandle(handle.Value) ?? string.Empty;

      // skip if we're inspecting our own process
      if (processName == "GlazeWM")
      {
        return;
      }

      // update values
      titleValue.Text = title;
      classNameValue.Text = className;
      processNameValue.Text = processName;
    }
  }
}
