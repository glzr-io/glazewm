using System.Windows.Forms;

namespace GlazeWM.Domain.Windows
{
  public partial class Inspector : Form
  {
    public Inspector()
    {
      InitializeComponent();

      MaximizeBox = false;
      MinimizeBox = false;
      FormBorderStyle = FormBorderStyle.FixedSingle;
    }

    private void titleCopyButton_Click(object sender, System.EventArgs e)
    {
      Clipboard.SetText(titleValue.Text);
    }

    private void classNameCopyButton_Click(object sender, System.EventArgs e)
    {
      Clipboard.SetText(classNameValue.Text);
    }

    private void processNameCopyButton_Click(object sender, System.EventArgs e)
    {
      Clipboard.SetText(processNameValue.Text);
    }
  }
}
