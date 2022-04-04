#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/kdev_t.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/device.h>
#include <linux/slab.h>                 //kmalloc()
#include <linux/uaccess.h>              //copy_to/from_user()
#include <linux/random.h>


MODULE_LICENSE("GPL");
MODULE_AUTHOR("Nobody");
MODULE_DESCRIPTION("Simple Device");

#define LOG(x) pr_info("Simple Device - " x);

struct request {
    char msg[10];
};

#define CMD_GIVE _IOW(0xF00BA6, 1, struct request)
#define CMD_TAKE _IOR(0xF00BA6, 2, struct request)

static dev_t dev_id;
static struct cdev simpledev_cdev;
static char storage[10] = {0};

static int      simpledev_open(struct inode *inode, struct file *file);
static int      simpledev_release(struct inode *inode, struct file *file);
static ssize_t  simpledev_read(struct file *filp, char __user *buf, size_t len,loff_t * off);
static ssize_t  simpledev_write(struct file *filp, const char *buf, size_t len, loff_t * off);
static long     simpledev_ioctl(struct file *filp, unsigned int cmd, unsigned long arg);

static struct file_operations fops =
{
    .owner          = THIS_MODULE,
    .read           = simpledev_read,
    .write          = simpledev_write,
    .open           = simpledev_open,
    .release        = simpledev_release,
    .unlocked_ioctl = simpledev_ioctl,
};

static int simpledev_open(struct inode *inode, struct file *file)
{
    LOG("file open()\n");
    return 0;
}

static int simpledev_release(struct inode *inode, struct file *file)
{
    LOG("file close()\n");
    return 0;
}

static ssize_t simpledev_read(struct file *filp, char __user *buf, size_t len, loff_t *off)
{
    char msg[] = "Hello From Simple Device!";
    size_t to_copy = sizeof(msg);

    LOG("file read()\n");

    if (len < to_copy) {
        to_copy = len;
    }

    return copy_to_user(buf, msg, to_copy);
}

static ssize_t simpledev_write(struct file *filp, const char __user *buf, size_t len, loff_t *off)
{
    char msg[11] = {0};
    size_t max_chars = 10;

    LOG("file write()\n");

    if (len < max_chars) {
        max_chars = len;
    }

    if (copy_from_user(msg, buf, max_chars)) {
        return -1;
    }

    pr_info("Simple Device - Received: %s", msg);
    return max_chars;
}

static long simpledev_ioctl(struct file *filp, unsigned int cmd, unsigned long arg) {
    struct request req;

    switch (cmd) {
        case CMD_GIVE:
            if (copy_from_user(&req, u64_to_user_ptr((u64)arg), sizeof(struct request))) {
                return -EINVAL;
            }
            memcpy(storage, req.msg, sizeof(req.msg));
            break;
        case CMD_TAKE:
            memcpy(req.msg, storage, sizeof(storage));
            if (copy_to_user(u64_to_user_ptr((u64)arg), &req, sizeof(struct request))) {
                return -EINVAL;
            }
            break;
        default:
            return -EINVAL;
    }

    return 0;
}

static int __init simpledev_init(void)
{
        LOG("alloc_chrdev_region()\n");
        if((alloc_chrdev_region(&dev_id, 0, 1, "simpledev")) < 0){
            pr_err("Simple Device Init - Cannot allocate major number\n");
            return -1;
        }

        LOG("cdev_init()\n");
        cdev_init(&simpledev_cdev, &fops);
        simpledev_cdev.owner = THIS_MODULE;

        LOG("cdev_add()\n");
        if((cdev_add(&simpledev_cdev, dev_id, 1)) < 0){
            pr_err("Simple Device Init - Cannot add the device to the system\n\n");
            unregister_chrdev_region(dev_id, 1);
            return -1;
        }

        LOG("Successfully initialized!\n");

        return 0;
}

static void __exit simpledev_exit(void)
{
    pr_info("Simple Device Exit!\n");
    cdev_del(&simpledev_cdev);
    unregister_chrdev_region(dev_id, 1);
}

module_init(simpledev_init);
module_exit(simpledev_exit);
