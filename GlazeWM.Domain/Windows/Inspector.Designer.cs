using System.Windows.Forms;

namespace GlazeWM.Domain.Windows
{
  partial class Inspector
  {
    /// <summary>
    /// Required designer variable.
    /// </summary>
    private System.ComponentModel.IContainer components = null;

    /// <summary>
    /// Clean up any resources being used.
    /// </summary>
    /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
    protected override void Dispose(bool disposing)
    {
      if (disposing && (components != null))
      {
        components.Dispose();
      }

      if (disposing && cursorSubscription != null)
      {
        cursorSubscription.Dispose();
      }

      base.Dispose(disposing);
    }

    #region Windows Form Designer generated code

    /// <summary>
    /// Required method for Designer support - do not modify
    /// the contents of this method with the code editor.
    /// </summary>
    private void InitializeComponent()
    {
      processNameLabel = new Label();
      classNameLabel = new Label();
      titleLabel = new Label();
      titleValue = new TextBox();
      classNameValue = new TextBox();
      processNameValue = new TextBox();
      SuspendLayout();
      // 
      // processNameLabel
      // 
      processNameLabel.AutoSize = true;
      processNameLabel.Font = new System.Drawing.Font("Segoe UI", 9F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point);
      processNameLabel.Location = new System.Drawing.Point(12, 78);
      processNameLabel.Name = "processNameLabel";
      processNameLabel.Size = new System.Drawing.Size(86, 15);
      processNameLabel.TabIndex = 0;
      processNameLabel.Text = "Process name:";
      // 
      // classNameLabel
      // 
      classNameLabel.AutoSize = true;
      classNameLabel.Font = new System.Drawing.Font("Segoe UI", 9F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point);
      classNameLabel.Location = new System.Drawing.Point(12, 49);
      classNameLabel.Name = "classNameLabel";
      classNameLabel.Size = new System.Drawing.Size(70, 15);
      classNameLabel.TabIndex = 1;
      classNameLabel.Text = "Class name:";
      // 
      // titleLabel
      // 
      titleLabel.AutoSize = true;
      titleLabel.Font = new System.Drawing.Font("Segoe UI", 9F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point);
      titleLabel.Location = new System.Drawing.Point(12, 20);
      titleLabel.Name = "titleLabel";
      titleLabel.Size = new System.Drawing.Size(35, 15);
      titleLabel.TabIndex = 2;
      titleLabel.Text = "Title:";
      // 
      // titleValue
      // 
      titleValue.Font = new System.Drawing.Font("Segoe UI", 9F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point);
      titleValue.Location = new System.Drawing.Point(104, 16);
      titleValue.Name = "titleValue";
      titleValue.ReadOnly = true;
      titleValue.Size = new System.Drawing.Size(296, 23);
      titleValue.TabIndex = 4;
      // 
      // classNameValue
      // 
      classNameValue.Font = new System.Drawing.Font("Segoe UI", 9F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point);
      classNameValue.Location = new System.Drawing.Point(104, 45);
      classNameValue.Name = "classNameValue";
      classNameValue.ReadOnly = true;
      classNameValue.Size = new System.Drawing.Size(296, 23);
      classNameValue.TabIndex = 5;
      // 
      // processNameValue
      // 
      processNameValue.Font = new System.Drawing.Font("Segoe UI", 9F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point);
      processNameValue.Location = new System.Drawing.Point(104, 74);
      processNameValue.Name = "processNameValue";
      processNameValue.ReadOnly = true;
      processNameValue.Size = new System.Drawing.Size(296, 23);
      processNameValue.TabIndex = 6;
      // 
      // Inspector
      // 
      AutoScaleDimensions = new System.Drawing.SizeF(7F, 15F);
      AutoScaleMode = AutoScaleMode.Font;
      BackColor = System.Drawing.SystemColors.Control;
      ClientSize = new System.Drawing.Size(414, 116);
      Controls.Add(processNameValue);
      Controls.Add(classNameValue);
      Controls.Add(titleValue);
      Controls.Add(titleLabel);
      Controls.Add(classNameLabel);
      Controls.Add(processNameLabel);
      Name = "Inspector";
      Text = "GlazeWM Inspector";
      ResumeLayout(false);
      PerformLayout();
    }

    #endregion

    private System.Windows.Forms.Label processNameLabel;
    private System.Windows.Forms.Label classNameLabel;
    private System.Windows.Forms.Label titleLabel;
    private System.Windows.Forms.TextBox titleValue;
    private System.Windows.Forms.TextBox classNameValue;
    private System.Windows.Forms.TextBox processNameValue;
  }
}
