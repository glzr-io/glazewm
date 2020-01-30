using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace LarsWM.Core
{
    class Program
    {
        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            Debug.WriteLine("Application started");

            //Application.Run();

            new Startup();

            // TODO: Read config file and initialise UserConfig class with its values
            // TODO: Register windows hooks
            // TODO: Create a workspace and assign a workspace to each connected display
            // TODO: Force initial layout
        }
    }
}
