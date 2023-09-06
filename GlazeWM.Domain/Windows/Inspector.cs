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
  }
}
