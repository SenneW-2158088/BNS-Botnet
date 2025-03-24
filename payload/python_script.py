import notify2


def hello_world():
    notify2.init("spam notifier")
    print("hello world")


hello_world()
