# bingpaper-desktop

拉取 bing 每日图片，设置桌面壁纸。

# 功能

从bing接口中，获取最新壁纸列表`get_wallpapers`。

界面左侧显示时间列表。

点击某个时间，主体区域显示图片，图片下面显示name和description。

提供一个下载按钮，可以下载图片`download_picture`到指定目录`get_home`。

提供一个设为壁纸按钮，将图片设置为当前壁纸`set_picture_for_screen`，screen使用`list_screens`的第一个。
