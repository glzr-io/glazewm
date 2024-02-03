using System;
using System.Reactive.Linq;
using System.Windows.Forms;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public partial class Inspector : Form
  {
    private readonly IDisposable _cursorSubscription;

    public Inspector()
    {
      InitializeComponent();

      MaximizeBox = false;
      MinimizeBox = false;
      FormBorderStyle = FormBorderStyle.FixedSingle;
      StartPosition = FormStartPosition.CenterParent;

      _cursorSubscription = MouseEvents.MouseMoves
        .Sample(TimeSpan.FromMilliseconds(50))
        .Subscribe(OnCursorMove);
    }

    private void OnCursorMove(MouseMoveEvent @event)
    {
      // get handle under cursor
      var handle = WindowFromPoint(@event.Point);

      // update the inspector info
      UpdateInspectorValues(handle);
    }

    public void UpdateInspectorValues(IntPtr? handle)
    {
      // skip if there is nothing to inspect
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
